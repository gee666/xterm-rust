// Copyright (c) 2020, 2023, 2026 The xterm.js authors. MIT license.
use crate::{
    raw_kitty_limit::{json, token, Activation},
    raw_sixel_limit::parse,
    support::*,
};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_test::*;
use xterm_web::image::ImageAddon;

pub(super) fn limit(pair: &Pair, rust: bool) -> JsValue {
    get(
        &if rust {
            pair.addon.option_inspection()
        } else {
            get(&get(&pair.source, "addon"), "_opts")
        },
        "pixelLimit",
    )
}
pub(super) fn pixel_pair(options: &str) -> Pair {
    // Fail old-code controls before opening or activating either terminal.
    let preflight = ImageAddon::new(eval("({pixelLimit:Symbol('preflight')})")).unwrap();
    preflight.dispose().unwrap();
    Pair::new(true, options)
}
pub(super) fn same_log(pair: &Pair, expected: &str) {
    for rust in [false, true] {
        assert_eq!(json(&get(&limit(pair, rust), "log")), expected);
    }
}
pub(super) fn set(value: &JsValue, key: &str, input: &JsValue) {
    js_sys::Reflect::set(value, &key.into(), input).unwrap();
}
pub(super) fn errors(pair: &Pair, text: &str) {
    let (actual, expected) = parse(pair, text);
    let actual = actual.unwrap_err();
    let expected = expected.unwrap_err();
    if token(&limit(pair, true)).is_undefined() {
        assert_eq!(get(&actual, "name"), get(&expected, "name"));
        assert_eq!(get(&actual, "message"), get(&expected, "message"));
    } else {
        assert_eq!(actual, token(&limit(pair, true)));
        assert_eq!(expected, token(&limit(pair, false)));
    }
}

#[wasm_bindgen_test(async)]
async fn raw_pixel_constructor_retains_values_without_coercion_in_all_bindings() {
    let direct = wasm_bindgen_futures::JsFuture::from(js_sys::Promise::resolve(&eval(
        "import('/suite.js')",
    )))
    .await
    .unwrap();
    let shipped = public_api().await;
    let run = js_sys::Function::new_with_args(
        "apis",
        r#"
      for(const api of apis){
        const values=[undefined,null,false,true,0,-0,NaN,Infinity,-Infinity,-1,1,
          '', '1000','no',0n,1000n,Symbol('pixel'),{},[],new Number(1000),
          document.all,()=>{}, {[Symbol.toPrimitive](){throw Error('eager')}},
          (()=>{const {proxy,revoke}=Proxy.revocable({},{});revoke();return proxy})()];
        for(const value of values){
          const addon=new api.ImageAddon({pixelLimit:value});
          try{
            for(const action of [()=>{},()=>addon.reset(),()=>addon.dispose()]){
              action();if(!Object.is(addon._opts.pixelLimit,value))throw Error('raw pixel identity');
            }
          }finally{addon.dispose();addon.free?.();}
        }
        let reads=0;
        const first=Symbol('first'),second={};
        const addon=new api.ImageAddon({get pixelLimit(){return ++reads===1?first:second}});
        try{
          if(reads!==2 || addon._opts.pixelLimit!==first)throw Error('copy order');
          addon.reset();
          if(reads!==2 || addon._opts.pixelLimit!==first)throw Error('reset changed pixel limit');
        }finally{addon.dispose();addon.free?.();}
      }
    "#,
    );
    run.call1(
        &JsValue::UNDEFINED,
        &js_sys::Array::of3(&eval("({ImageAddon:SourceImageAddon})"), &direct, &shipped),
    )
    .unwrap();
}

#[wasm_bindgen_test(async)]
async fn raw_pixel_geometry_repeats_comparison_and_sqrt_at_source_phases() {
    let pair=pixel_pair("({sixelSupport:false,showPlaceholder:false,pixelLimit:{log:[],n:0,[Symbol.toPrimitive](h){this.log.push(h);return ++this.n===1?0:100}}})");
    for text in ["\x1b[c", "\x1b[?2;3S", "\x1b[?1;4S"] {
        pair.write(text).await;
        pair.compare();
    }
    same_log(&pair, "[]");
    pair.write("\x1b[?2;1S").await;
    pair.compare();
    same_log(&pair, "[\"number\",\"number\"]");
    assert_eq!(
        String::from_utf16(pair.replies.borrow().last().unwrap()).unwrap(),
        "\x1b[?2;0;10;10S"
    );
    pair.write("\x1b[?2;4S").await;
    pair.compare();
    same_log(&pair, "[\"number\",\"number\",\"number\"]");
    // Before .open(), source falls back to terminal columns/rows.
    let closed = Pair::new(false, "({sixelSupport:false,pixelLimit:'100'})");
    closed.write("\x1b[?2;1S\x1b[?2;4S").await;
    closed.compare();
    for dispose in [false, true] {
        let pair = Rc::new(pixel_pair("({sixelSupport:false,pixelLimit:{callback:null,log:[],n:0,[Symbol.toPrimitive](h){this.log.push(h);this.callback();return ++this.n===1?0:100}}})"));
        let weak = Rc::downgrade(&pair);
        let callback = Closure::<dyn Fn()>::new(move || {
            let pair = weak.upgrade().unwrap();
            if dispose {
                pair.addon.dispose().unwrap();
            } else {
                pair.addon.reset().unwrap();
            }
        });
        set(&limit(&pair, true), "callback", callback.as_ref());
        let source_callback =
            js_sys::Function::new_with_args("a,d", "return ()=>d?a.dispose():a.reset()")
                .call2(
                    &JsValue::UNDEFINED,
                    &get(&pair.source, "addon"),
                    &dispose.into(),
                )
                .unwrap();
        set(&limit(&pair, false), "callback", &source_callback);
        let (actual, expected) = parse(&pair, "\x1b[?2;1S");
        actual.unwrap();
        expected.unwrap();
        pair.compare();
        same_log(&pair, "[\"number\",\"number\"]");
        assert_eq!(pair.replies.borrow().len(), 1);
        for rust in [false, true] {
            set(&limit(&pair, rust), "callback", &JsValue::NULL);
        }
    }
}

#[wasm_bindgen_test(async)]
async fn raw_pixel_geometry_primitives_and_exact_errors_match_source() {
    for value in [
        "undefined",
        "null",
        "false",
        "true",
        "0",
        "-0",
        "NaN",
        "Infinity",
        "-Infinity",
        "-1",
        "'100'",
        "'no'",
        "[]",
        "[100]",
        "new Number(100)",
        "1000000000n",
    ] {
        let pair = pixel_pair(&format!("({{sixelSupport:false,pixelLimit:{value}}})"));
        pair.write("\x1b[?2;1S").await;
        pair.compare();
        if value != "1000000000n" {
            pair.write("\x1b[?2;4S").await;
            pair.compare();
        } else {
            errors(&pair, "\x1b[?2;4S");
        }
    }
    for value in [
        "Symbol('pixel')",
        "1n",
        "({token:{},[Symbol.toPrimitive](){throw this.token}})",
        "({[Symbol.toPrimitive](){return {}}})",
    ] {
        for request in ["\x1b[?2;1S", "\x1b[?2;4S"] {
            let pair = pixel_pair(&format!("({{sixelSupport:false,pixelLimit:{value}}})"));
            errors(&pair, request);
            assert!(pair.replies.borrow().is_empty());
        }
    }
}

#[wasm_bindgen_test]
fn raw_pixel_sixel_activation_converts_synchronously_and_preserves_errors() {
    for expression in [
        "Symbol('pixel')",
        "1000n",
        "({token:{},[Symbol.toPrimitive](){throw this.token}})",
        "({token:{},valueOf(){throw this.token}})",
        "({[Symbol.toPrimitive](){return {}}})",
    ] {
        let trial = Activation::with_key(
            &format!("({{showPlaceholder:false,pixelLimit:{expression}}})"),
            "pixelLimit",
        );
        let (actual, expected) = trial.activate();
        let actual = actual.unwrap_err();
        let expected = expected.unwrap_err();
        if token(&trial.value(true)).is_undefined() {
            assert_eq!(get(&actual, "name"), get(&expected, "name"));
            assert_eq!(get(&actual, "message"), get(&expected, "message"));
        } else {
            assert_eq!(actual, token(&trial.value(true)));
            assert_eq!(expected, token(&trial.value(false)));
        }
        trial.compare_membership();
        assert_eq!(
            get(&trial.addon.handler_inspection(), "size"),
            JsValue::from(0)
        );
    }
}

#[wasm_bindgen_test(async)]
async fn raw_pixel_sixel_memory_capture_is_once_per_activation_not_per_packet() {
    let pair=pixel_pair("({showPlaceholder:false,pixelLimit:{log:[],[Symbol.toPrimitive](h){this.log.push(h);return 16777216}}})");
    same_log(&pair, "[\"number\"]");
    pair.ready().await;
    for text in [sixel(), "\x1b[?1;2S", sixel()] {
        pair.write(text).await;
        pair.compare();
    }
    same_log(&pair, "[\"number\"]");
    pair.compare_image(0.0, 0.0);
    for value in ["'16777216'", "[16777216]", "new Number(16777216)"] {
        let pair = pixel_pair(&format!("({{showPlaceholder:false,pixelLimit:{value}}})"));
        pair.ready().await;
        pair.write(sixel()).await;
        pair.compare();
        pair.compare_image(0.0, 0.0);
    }
}

#[wasm_bindgen_test]
fn raw_pixel_sixel_activation_conversion_can_reset_or_dispose() {
    for dispose in [false, true] {
        let trial=Rc::new(Activation::with_key("({showPlaceholder:false,pixelLimit:{callback:null,log:[],[Symbol.toPrimitive](h){this.log.push(h);this.callback();return 16777216}}})","pixelLimit"));
        let weak = Rc::downgrade(&trial);
        let callback = Closure::<dyn Fn()>::new(move || {
            let trial = weak.upgrade().unwrap();
            if dispose {
                trial.addon.dispose().unwrap()
            } else {
                trial.addon.reset().unwrap();
            }
        });
        set(&trial.value(true), "callback", callback.as_ref());
        let source_callback =
            js_sys::Function::new_with_args("a,d", "return ()=>d?a.dispose():a.reset()")
                .call2(
                    &JsValue::UNDEFINED,
                    &get(&trial.source, "addon"),
                    &dispose.into(),
                )
                .unwrap();
        set(&trial.value(false), "callback", &source_callback);
        let (actual, expected) = trial.activate();
        actual.unwrap();
        expected.unwrap();
        trial.compare_membership();
        for rust in [true, false] {
            assert_eq!(json(&get(&trial.value(rust), "log")), "[\"number\"]");
            set(&trial.value(rust), "callback", &JsValue::NULL);
        }
    }
}
