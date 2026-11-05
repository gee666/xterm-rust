// Copyright (c) 2026 The xterm.js authors. MIT license.
use crate::{
    raw_kitty_limit::{json, token, Activation},
    support::*,
};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_test::*;
use xterm_web::image::ImageAddon;

#[wasm_bindgen_test]
fn raw_iip_limit_constructor_retains_identity_and_two_copy_order() {
    for expression in [
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
        "''",
        "'1000'",
        "'no'",
        "0n",
        "1000n",
        "Symbol('size')",
        "{}",
        "[]",
        "new Number(1000)",
    ] {
        let value = eval(expression);
        let options = js_sys::Object::new();
        js_sys::Reflect::set(&options, &"iipSizeLimit".into(), &value).unwrap();
        let actual = ImageAddon::new(options.clone().into()).unwrap();
        let expected = js_sys::Function::new_with_args("o", "return new SourceImageAddon(o)")
            .call1(&JsValue::UNDEFINED, &options)
            .unwrap();
        for addon in [actual.option_inspection(), get(&expected, "_opts")] {
            assert!(js_sys::Object::is(&get(&addon, "iipSizeLimit"), &value));
        }
        actual.reset().unwrap();
        actual.dispose().unwrap();
        assert!(js_sys::Object::is(
            &get(&actual.option_inspection(), "iipSizeLimit"),
            &value
        ));
        call(&expected, "dispose", &[]);
    }
    let trial = Activation::with_key("(()=>{let reads=0;const first={log:[],[Symbol.toPrimitive](hint){this.log.push(hint);return 1000}},second={[Symbol.toPrimitive](){throw Error('second')}};return {sixelSupport:false,kittySupport:false,showPlaceholder:false,get iipSizeLimit(){return ++reads===1?first:second}}})()", "iipSizeLimit");
    for rust in [false, true] {
        assert_eq!(json(&get(&trial.value(rust), "log")), "[]");
    }
    for count in 1..=2 {
        let (actual, expected) = trial.activate();
        actual.unwrap();
        expected.unwrap();
        trial.compare_membership();
        for rust in [false, true] {
            assert_eq!(
                get(&get(&trial.value(rust), "log"), "length"),
                JsValue::from(count)
            );
            assert_eq!(
                get(&get(&trial.value(rust), "log"), "0"),
                JsValue::from("number")
            );
        }
        trial.addon.reset().unwrap();
        call(&get(&trial.source, "addon"), "reset", &[]);
    }
}

#[wasm_bindgen_test(async)]
async fn raw_iip_limit_disabled_never_converts_or_decodes() {
    for expression in [
        "Symbol('size')",
        "1000n",
        "({[Symbol.toPrimitive](){throw Error('disabled')}})",
    ] {
        let options = format!("({{sixelSupport:false,iipSupport:false,kittySupport:false,showPlaceholder:false,iipSizeLimit:{expression}}})");
        // Fail before terminal allocation when running the old-code control.
        let preflight = ImageAddon::new(eval(&options)).unwrap();
        preflight.dispose().unwrap();
        let pair = Pair::new(false, &options);
        pair.write(iip()).await;
        pair.compare();
        pair.addon.reset().unwrap();
        call(&get(&pair.source, "addon"), "reset", &[]);
        pair.compare();
    }
}

#[wasm_bindgen_test]
fn raw_iip_limit_activation_errors_preserve_prior_work_and_skip_kitty() {
    for expression in [
        "Symbol('size')",
        "1000n",
        "Infinity",
        "4000000000",
        "({token:{},[Symbol.toPrimitive](){throw this.token}})",
        "({token:{},valueOf(){throw this.token}})",
        "({[Symbol.toPrimitive](){return {}}})",
    ] {
        // SIXEL has registered before IIP conversion; Kitty has not started.
        let trial = Activation::with_key(
            &format!("({{showPlaceholder:false,iipSizeLimit:{expression}}})"),
            "iipSizeLimit",
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
            JsValue::from(1)
        );
    }
}

#[wasm_bindgen_test]
fn raw_iip_limit_nonfinite_and_negative_settings_are_only_constructed() {
    // Source accepts these decoder settings at construction, not necessarily
    // at memory initialization. Do not execute invalid-memory or hanging paths.
    for expression in ["undefined", "NaN", "-Infinity", "-1", "'no'", "{}"] {
        let trial = Activation::with_key(&format!("({{sixelSupport:false,kittySupport:false,showPlaceholder:false,iipSizeLimit:{expression}}})"), "iipSizeLimit");
        let (actual, expected) = trial.activate();
        actual.unwrap();
        expected.unwrap();
        trial.compare_membership();
        assert_eq!(
            get(&trial.addon.handler_inspection(), "size"),
            JsValue::from(1)
        );
    }
}

#[wasm_bindgen_test]
fn raw_iip_limit_conversion_can_reset_or_dispose_before_registration() {
    for dispose in [false, true] {
        let trial = Rc::new(Activation::with_key("({sixelSupport:false,kittySupport:false,showPlaceholder:false,iipSizeLimit:{callback:null,log:[],[Symbol.toPrimitive](hint){this.log.push(hint);this.callback();return 1000}}})", "iipSizeLimit"));
        let weak = Rc::downgrade(&trial);
        let callback = Closure::<dyn Fn()>::new(move || {
            let trial = weak.upgrade().unwrap();
            if dispose {
                trial.addon.dispose().unwrap();
            } else {
                trial.addon.reset().unwrap();
            }
        });
        js_sys::Reflect::set(&trial.value(true), &"callback".into(), callback.as_ref()).unwrap();
        let source_callback = js_sys::Function::new_with_args(
            "a,dispose",
            "return ()=>dispose?a.dispose():a.reset()",
        )
        .call2(
            &JsValue::UNDEFINED,
            &get(&trial.source, "addon"),
            &dispose.into(),
        )
        .unwrap();
        js_sys::Reflect::set(&trial.value(false), &"callback".into(), &source_callback).unwrap();
        let (actual, expected) = trial.activate();
        actual.unwrap();
        expected.unwrap();
        trial.compare_membership();
        for rust in [false, true] {
            assert_eq!(json(&get(&trial.value(rust), "log")), "[\"number\"]");
            js_sys::Reflect::set(&trial.value(rust), &"callback".into(), &JsValue::NULL).unwrap();
        }
    }
}

#[wasm_bindgen_test(async)]
async fn raw_iip_limit_finite_numeric_forms_enforce_stream_threshold_and_pixels() {
    for (expression, inserted) in [
        ("null", false),
        ("false", false),
        ("true", false),
        ("0", false),
        ("-0", false),
        ("1.5", false),
        ("'41'", false),
        ("'41.5'", true),
        ("'42'", true),
        ("'1000'", true),
        ("[]", false),
        ("[1000]", true),
        ("new Number(1000)", true),
        ("({valueOf(){return 1000}})", true),
    ] {
        let options = format!("({{sixelSupport:false,kittySupport:false,showPlaceholder:false,iipSizeLimit:{expression}}})");
        let pair = Pair::new(true, &options);
        pair.write(iip()).await;
        pair.compare();
        assert_eq!(
            get(&pair.events, "count"),
            JsValue::from(u32::from(inserted)),
            "{expression}"
        );
        if inserted {
            pair.compare_image(0.0, 0.0);
        }
    }
}
