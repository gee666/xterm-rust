// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Catch live event getters and coercion failures before Rust guards unwind.
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::Event;

// These helpers are numeric FFI, not terminal policy. Static imports avoid CSP
// dependent eval and preserve JavaScript primitive/valueOf exception semantics.
#[wasm_bindgen(inline_js = r#"
export function xtermMouseNumber(v) { return +v; }
export function xtermMouseMultiply(v, n) { return v * n; }
export function xtermMouseDivide(v, n) { return v / n; }
export function xtermMouseLess(v, n) { return v < n; }
export function xtermMouseBit(v, n) { return (v & n) !== 0; }
export function xtermMouseProperty(v, k) { return v[k]; }
export function xtermMouseWindow() { return window; }
export function xtermMousePadding(v) { return parseInt(v, 10); }
export function xtermMouseSubtract(a, b) { return a - b; }
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = xtermMouseNumber)]
    pub(super) fn number_value(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = xtermMouseMultiply)]
    pub(super) fn multiply(value: &JsValue, factor: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = xtermMouseDivide)]
    pub(super) fn divide(value: &JsValue, divisor: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = xtermMouseLess)]
    pub(super) fn less(value: &JsValue, limit: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = xtermMouseProperty)]
    fn property(value: &JsValue, key: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = xtermMouseWindow)]
    pub(super) fn global_window() -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = xtermMousePadding)]
    pub(super) fn padding(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = xtermMouseSubtract)]
    pub(super) fn subtract(left: &JsValue, right: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = xtermMouseBit)]
    fn bit_value(value: &JsValue, mask: i32) -> Result<bool, JsValue>;
}

pub(super) fn get(event: &Event, name: &str) -> Result<JsValue, JsValue> {
    get_js(event, name)
}

pub(super) fn get_js(value: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    property(value, &JsValue::from_str(name))
}

pub(super) fn number(event: &Event, name: &str) -> Result<f64, JsValue> {
    number_value(&get(event, name)?)
}

pub(super) fn truth(event: &Event, name: &str) -> Result<bool, JsValue> {
    Ok(get(event, name)?.is_truthy())
}

pub(super) fn strict_zero(value: &JsValue) -> bool {
    value.as_f64() == Some(0.0)
}

pub(super) fn bit(event: &Event, name: &str, mask: i32) -> Result<bool, JsValue> {
    bit_value(&get(event, name)?, mask)
}

pub(super) fn call(event: &JsValue, name: &str, args: &js_sys::Array) -> Result<JsValue, JsValue> {
    let function = get_js(event, name)?;
    // Reflect.apply performs the callable check and returns its original JS
    // TypeError for a non-function. No Function method is invoked via this view.
    js_sys::Reflect::apply(function.unchecked_ref(), event, args)
}

pub(super) fn prevent(event: &Event) -> Result<(), JsValue> {
    call(event, "preventDefault", &js_sys::Array::new())?;
    Ok(())
}

pub(super) fn cancel(event: &Event) -> Result<(), JsValue> {
    prevent(event)?;
    call(event, "stopPropagation", &js_sys::Array::new())?;
    Ok(())
}

pub(super) fn alt_modifier(event: &Event) -> Result<bool, JsValue> {
    let args = js_sys::Array::new();
    args.push(&JsValue::from_str("Alt"));
    Ok(call(event, "getModifierState", &args)?.is_truthy())
}
