#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use js_sys::{JsString, Object, Promise, Reflect, TypeError};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_core::{headless::HeadlessError, terminal::TerminalError};
use xterm_web::{
    js_error::to_js_error, terminal::Terminal, utility_addons::bindings::ClipboardAddon,
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn default_base64_uses_text_encoder_argument_coercion() {
    let codec = xterm_web::utility_addons::bindings::Base64::new();
    for (value, expected) in [
        (JsValue::UNDEFINED, ""),
        (JsValue::NULL, "bnVsbA=="),
        (JsValue::FALSE, "ZmFsc2U="),
        (JsValue::from_f64(42.0), "NDI="),
    ] {
        assert_eq!(
            codec.encode_text(value).unwrap().as_string().unwrap(),
            expected
        );
    }
    assert!(codec
        .encode_text(js_sys::Symbol::new(None).into())
        .unwrap_err()
        .is_instance_of::<TypeError>());
}
#[wasm_bindgen_test]
fn custom_codec_receives_provider_object_and_interpolates_raw_encoded_value() {
    let raw = Object::new();
    let value = raw.clone();
    let provider = Object::new();
    let read = Closure::wrap(
        Box::new(move |_: JsString| -> JsValue { value.clone().into() })
            as Box<dyn FnMut(JsString) -> JsValue>,
    );
    Reflect::set(&provider, &"readText".into(), read.as_ref()).unwrap();
    let codec = Object::new();
    let expected = raw.clone();
    let encode = Closure::wrap(Box::new(move |value: JsValue| -> JsValue {
        assert!(Object::is(&value, expected.as_ref()));
        JsValue::from_f64(42.0)
    }) as Box<dyn FnMut(JsValue) -> JsValue>);
    Reflect::set(&codec, &"encodeText".into(), encode.as_ref()).unwrap();
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let replies = Rc::new(RefCell::new(Vec::new()));
    let capture = replies.clone();
    terminal
        .shared_native()
        .on_data()
        .subscribe(move |data| capture.borrow_mut().push(data.clone()));
    let addon = ClipboardAddon::new(codec.into(), provider.into());
    addon.activate(&terminal).unwrap();
    terminal.shared_native().write("\x1b]52;c;?\x07").unwrap();
    assert_eq!(
        String::from_utf16(&replies.borrow()[0]).unwrap(),
        "\x1b]52;c;42\x07"
    );
    addon.dispose();
    terminal.dispose().unwrap();
}
#[wasm_bindgen_test]
fn custom_decoded_object_reaches_provider_without_string_coercion() {
    let raw = Object::new();
    let value = raw.clone();
    let codec = Object::new();
    let decode = Closure::wrap(
        Box::new(move |_: JsString| -> JsValue { value.clone().into() })
            as Box<dyn FnMut(JsString) -> JsValue>,
    );
    Reflect::set(&codec, &"decodeText".into(), decode.as_ref()).unwrap();
    let provider = Object::new();
    let expected = raw.clone();
    let write = Closure::wrap(Box::new(move |selection: JsString, value: JsValue| {
        assert_eq!(selection.as_string().unwrap(), "p");
        assert!(Object::is(&value, expected.as_ref()));
    }) as Box<dyn FnMut(JsString, JsValue)>);
    Reflect::set(&provider, &"writeText".into(), write.as_ref()).unwrap();
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let addon = ClipboardAddon::new(codec.into(), provider.into());
    addon.activate(&terminal).unwrap();
    terminal.shared_native().write("\x1b]52;p;raw\x07").unwrap();
    addon.dispose();
    terminal.dispose().unwrap();
}
#[wasm_bindgen_test]
fn noncallable_provider_method_throws_type_error_not_the_property_value() {
    let provider = Object::new();
    Reflect::set(&provider, &"readText".into(), &JsValue::from_f64(1.0)).unwrap();
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let addon = ClipboardAddon::new(JsValue::UNDEFINED, provider.into());
    addon.activate(&terminal).unwrap();
    let error = terminal
        .shared_native()
        .write("\x1b]52;c;?\x07must-not-print")
        .unwrap_err();
    assert!(to_js_error(&error).is_instance_of::<TypeError>());
    addon.dispose();
    terminal.dispose().unwrap();
}
#[wasm_bindgen_test(async)]
async fn pending_query_uses_the_terminal_activated_when_provider_settles() {
    let provider = Object::new();
    let read = Closure::wrap(Box::new(move |_: JsString| -> JsValue {
        Promise::resolve(&JsString::from("hello")).into()
    }) as Box<dyn FnMut(JsString) -> JsValue>);
    Reflect::set(&provider, &"readText".into(), read.as_ref()).unwrap();
    let first = Terminal::new(JsValue::UNDEFINED).unwrap();
    let second = Terminal::new(JsValue::UNDEFINED).unwrap();
    let replies = Rc::new(RefCell::new(Vec::new()));
    let capture = replies.clone();
    second
        .shared_native()
        .on_data()
        .subscribe(move |data| capture.borrow_mut().push(data.clone()));
    let addon = ClipboardAddon::new(JsValue::UNDEFINED, provider.into());
    addon.activate(&first).unwrap();
    assert!(matches!(
        first.shared_native().write("\x1b]52;c;?\x07"),
        Err(HeadlessError::Engine(TerminalError::Pending))
    ));
    let pending = first.shared_native().take_pending().unwrap().unwrap();
    addon.activate(&second).unwrap();
    first
        .shared_native()
        .resume(pending.resolve().await.unwrap())
        .unwrap();
    assert_eq!(
        String::from_utf16(&replies.borrow()[0]).unwrap(),
        "\x1b]52;c;aGVsbG8=\x07"
    );
    addon.dispose();
    first.dispose().unwrap();
    second.dispose().unwrap();
}
