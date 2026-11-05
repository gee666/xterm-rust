// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use crate::support::*;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::image::ImageAddon;

const FLAGS: [&str; 5] = [
    "enableSizeReports",
    "sixelSupport",
    "sixelScrolling",
    "iipSupport",
    "kittySupport",
];

fn source(options: &JsValue) -> JsValue {
    js_sys::Function::new_with_args("options", "return new SourceImageAddon(options)")
        .call1(&JsValue::UNDEFINED, options)
        .unwrap()
}
fn identical(actual: &JsValue, expected: &JsValue) {
    assert!(js_sys::Object::is(actual, expected));
}
fn compare_flags(rust: &ImageAddon, source: &JsValue) {
    for key in FLAGS {
        identical(
            &get(&rust.option_inspection(), key),
            &get(&get(source, "_opts"), key),
        );
    }
}

#[wasm_bindgen_test]
fn raw_flag_values_retain_identity_without_primitive_conversion() {
    for expression in [
        "undefined", "null", "false", "true", "0", "-0", "NaN",
        "Infinity", "-1", "''", "'false'", "[]", "[0]", "({})",
        "0n", "1n", "Symbol('flag')",
        "({[Symbol.toPrimitive](){throw Error('unexpected conversion')}})",
        "({valueOf(){throw Error('unexpected valueOf')},toString(){throw Error('unexpected toString')}})",
    ] {
        let value = eval(expression);
        let options = js_sys::Object::new();
        for key in FLAGS {
            js_sys::Reflect::set(&options, &key.into(), &value).unwrap();
        }
        let rust = ImageAddon::new(options.clone().into()).unwrap();
        let source = source(&options);
        let retained = rust.option_inspection();
        compare_flags(&rust, &source);
        for key in FLAGS {
            identical(&get(&retained, key), &value);
        }
        rust.reset().unwrap();
        call(&source, "reset", &[]);
        compare_flags(&rust, &source);
        rust.dispose().unwrap();
        call(&source, "dispose", &[]);
        drop(rust);
        for key in FLAGS {
            identical(&get(&retained, key), &value);
        }
    }
}

#[wasm_bindgen_test]
fn raw_flag_getters_copy_twice_but_reset_only_replaces_scrolling() {
    let make = || {
        eval(
            r#"(()=>{
      const first=Symbol('first'),second={tag:'second'},counts={};
      const options={};
      for(const key of ['enableSizeReports','sixelSupport','sixelScrolling','iipSupport','kittySupport']){
        counts[key]=0;
        Object.defineProperty(options,key,{enumerable:true,get(){return ++counts[key]===1?first:second}});
      }
      return {options,first,second,counts};
    })()"#,
        )
    };
    let input = make();
    let expected = make();
    let rust = ImageAddon::new(get(&input, "options")).unwrap();
    let source = source(&get(&expected, "options"));
    for key in FLAGS {
        assert_eq!(get(&get(&input, "counts"), key), JsValue::from(2));
        assert_eq!(get(&get(&expected, "counts"), key), JsValue::from(2));
        identical(&get(&rust.option_inspection(), key), &get(&input, "first"));
        identical(&get(&get(&source, "_opts"), key), &get(&expected, "first"));
    }
    for _ in 0..2 {
        rust.reset().unwrap();
        call(&source, "reset", &[]);
        for key in FLAGS {
            let copy = if key == "sixelScrolling" {
                "second"
            } else {
                "first"
            };
            identical(&get(&rust.option_inspection(), key), &get(&input, copy));
            identical(&get(&get(&source, "_opts"), key), &get(&expected, copy));
            assert_eq!(get(&get(&input, "counts"), key), JsValue::from(2));
        }
    }
    rust.dispose().unwrap();
    call(&source, "dispose", &[]);
}

#[wasm_bindgen_test(async)]
async fn raw_scrolling_controls_and_resets_preserve_real_sixel_state_and_pixels() {
    for expression in [
        "undefined",
        "null",
        "0",
        "''",
        "'false'",
        "[]",
        "Symbol('scroll')",
        "({[Symbol.toPrimitive](){throw Error('must not convert')}})",
    ] {
        let pair = Pair::new(
            true,
            &format!("({{showPlaceholder:false,sixelScrolling:{expression}}})"),
        );
        pair.ready().await;
        let original = get(&get(&get(&pair.source, "addon"), "_opts"), "sixelScrolling");
        let view = pair.addon.option_inspection();
        let retained = get(&view, "sixelScrolling");
        for ignored in [
            "\u{1b}[?81h",
            "\u{1b}[?80:1h",
            "\u{1b}[?1;4S",
            "\u{1b}[?1;2S",
        ] {
            pair.write(ignored).await;
            identical(&get(&view, "sixelScrolling"), &retained);
            identical(
                &get(&get(&get(&pair.source, "addon"), "_opts"), "sixelScrolling"),
                &original,
            );
        }
        for control in ["", "\x1b[?80h", "\x1b[?80l", "\x1b[!p", "\x1bc"] {
            pair.write(control).await;
            let source = get(&pair.source, "addon");
            // Objects/symbols are independently constructed on each side.
            if control.is_empty() || control == "\x1b[!p" || control == "\x1bc" {
                identical(&get(&view, "sixelScrolling"), &retained);
                identical(&get(&get(&source, "_opts"), "sixelScrolling"), &original);
            } else {
                identical(
                    &get(&view, "sixelScrolling"),
                    &get(&get(&source, "_opts"), "sixelScrolling"),
                );
            }
            pair.write("\x1b[3;1H").await;
            pair.write(sixel()).await;
            pair.compare();
            let mut images = 0;
            for y in 0..10 {
                let actual = pair
                    .addon
                    .get_image_at_buffer_cell(0.0, f64::from(y))
                    .unwrap();
                let expected = call(&source, "getImageAtBufferCell", &[0.into(), y.into()]);
                assert_eq!(actual.is_undefined(), expected.is_undefined());
                if !actual.is_undefined() {
                    images += 1;
                    assert_eq!(pixels(&actual), pixels(&expected));
                }
            }
            assert!(images > 0);
        }
    }
}

#[wasm_bindgen_test(async)]
async fn raw_support_flags_activate_by_truthiness_without_conversion() {
    for expression in [
        "undefined",
        "null",
        "0",
        "NaN",
        "''",
        "'false'",
        "[]",
        "0n",
        "1n",
        "Symbol('flag')",
        "({valueOf(){throw Error('unexpected')},toString(){throw Error('unexpected')}})",
    ] {
        let pair = Pair::new(
            false,
            &format!(
                r#"({{
          showPlaceholder:false,enableSizeReports:{expression},
          sixelSupport:{expression},iipSupport:{expression},kittySupport:{expression}
        }})"#
            ),
        );
        pair.ready().await;
        let source = get(&pair.source, "addon");
        for key in ["sixel", "iip", "kitty"] {
            let actual = call(&pair.addon.handler_inspection(), "get", &[key.into()]);
            let expected = call(&get(&source, "_handlers"), "get", &[key.into()]);
            assert_eq!(actual.is_undefined(), expected.is_undefined());
        }
        let actual = get(&pair.terminal.options(), "windowOptions");
        let expected = get(
            &get(&get(&pair.source, "terminal"), "options"),
            "windowOptions",
        );
        for key in ["getWinSizePixels", "getCellSizePixels", "getWinSizeChars"] {
            assert_eq!(get(&actual, key), get(&expected, key));
        }
        pair.write("\x1b[c").await;
        pair.compare();
    }
}

// A reset callback sees the replacement raw default before any image disposal.
#[wasm_bindgen_test(async)]
async fn image_added_reset_reentry_restores_second_raw_scrolling_copy() {
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    let pair = Rc::new(Pair::new(
        true,
        r#"(()=>{
      let reads=0;return {showPlaceholder:false,
        get sixelScrolling(){return ++reads===1?'initial':0}}
    })()"#,
    ));
    pair.ready().await;
    let weak = Rc::downgrade(&pair);
    let observed = Rc::new(std::cell::RefCell::new(Vec::new()));
    let capture = observed.clone();
    let callback = Closure::wrap(Box::new(move || {
        let pair = weak.upgrade().unwrap();
        pair.addon.reset().unwrap();
        capture
            .borrow_mut()
            .push(get(&pair.addon.option_inspection(), "sixelScrolling"));
    }) as Box<dyn FnMut()>);
    let registration = pair
        .addon
        .on_image_added()
        .call1(&JsValue::UNDEFINED, callback.as_ref().unchecked_ref())
        .unwrap();
    call(
        &get(&pair.source, "addon"),
        "onImageAdded",
        &[js_sys::Function::new_no_args(
            "this.reset();if(this._opts.sixelScrolling!==0)throw Error('wrong copy')",
        )
        .bind0(&get(&pair.source, "addon"))
        .into()],
    );
    pair.write(sixel()).await;
    pair.compare();
    compare_flags(&pair.addon, &get(&pair.source, "addon"));
    pair.write("\x1b[?80l").await;
    pair.write(sixel()).await;
    pair.compare();
    assert_eq!(pair.addon.storage_usage().unwrap(), 0.0);
    assert_eq!(observed.borrow().len(), 2);
    for value in observed.borrow().iter() {
        identical(value, &0.into());
    }
    call(&registration, "dispose", &[]);
}
