// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::HtmlElement;
pub fn units(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}
pub fn attribute(element: &HtmlElement, name: &str) -> Vec<u16> {
    let get = Reflect::get(element, &"getAttribute".into()).unwrap();
    assert!(get.is_function());
    let value = get
        .unchecked_into::<Function>()
        .call1(element, &name.into())
        .unwrap();
    assert!(value.is_string());
    xterm_web::utf16::from_js_string(&value.unchecked_into())
}
pub fn text(element: &HtmlElement) -> Vec<u16> {
    let value = js_sys::Reflect::get(element, &JsValue::from_str("textContent")).unwrap();
    assert!(value.is_string());
    xterm_web::utf16::from_js_string(&value.unchecked_into())
}
