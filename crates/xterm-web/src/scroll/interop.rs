//! JavaScript property/call/coercion primitives for DOM adapters. Algorithms,
//! state transitions and scroll calculations stay in Rust.
use crate::terminal::options::{error_to_js, from_js, into_js};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function scrollGet(value, key) { return value[key]; }
export function scrollSet(value, key, next) { 'use strict'; value[key] = next; }
export function scrollMethod0(value, key) { return value[key](); }
export function scrollMethod1(value, key, a) { return value[key](a); }
export function scrollMethod2(value, key, a, b) { return value[key](a,b); }
export function scrollSubtract(a,b) { return a-b; }
export function scrollMultiply(a,b) { return a*b; }
export function scrollDivide(a,b) { return a/b; }
export function scrollNegate(value) { return -value; }
export function scrollClock() { return Date.now(); }
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = scrollGet)]
    pub fn get(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = scrollSet)]
    pub fn set(value: &JsValue, key: &str, next: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = scrollMethod0)]
    pub fn method0(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = scrollMethod1)]
    pub fn method1(value: &JsValue, key: &str, a: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = scrollMethod2)]
    pub fn method2(
        value: &JsValue,
        key: &str,
        a: &JsValue,
        b: &JsValue,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn scrollSubtract(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn scrollMultiply(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn scrollDivide(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn scrollNegate(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = scrollClock)]
    pub fn clock() -> Result<JsValue, JsValue>;
}
pub fn nullish(value: &JsValue) -> bool {
    value.is_null() || value.is_undefined()
}
pub fn number(value: &JsValue) -> Result<f64, JsValue> {
    from_js(value)?
        .to_number()
        .map_err(|error| error_to_js(&error))
}
pub fn add(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue> {
    into_js(
        &from_js(a)?
            .js_add(&from_js(b)?)
            .map_err(|error| error_to_js(&error))?,
    )
}
pub fn subtract(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue> {
    match (a.as_f64(), b.as_f64()) {
        (Some(a), Some(b)) => Ok((a - b).into()),
        _ => scrollSubtract(a, b),
    }
}
pub fn multiply(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue> {
    match (a.as_f64(), b.as_f64()) {
        (Some(a), Some(b)) => Ok((a * b).into()),
        _ => scrollMultiply(a, b),
    }
}
pub fn divide(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue> {
    match (a.as_f64(), b.as_f64()) {
        (Some(a), Some(b)) => Ok((a / b).into()),
        _ => scrollDivide(a, b),
    }
}
pub fn negate(value: &JsValue) -> Result<JsValue, JsValue> {
    match value.as_f64() {
        Some(value) => Ok((-value).into()),
        None => scrollNegate(value),
    }
}
pub fn event_window(event: &JsValue, fallback: &web_sys::Window) -> Result<JsValue, JsValue> {
    let owner = if nullish(event) {
        JsValue::UNDEFINED
    } else {
        get(event, "ownerDocument")?
    };
    let view = if nullish(&owner) {
        JsValue::UNDEFINED
    } else {
        get(&owner, "defaultView")?
    };
    if view.is_truthy() {
        return get(&get(event, "ownerDocument")?, "defaultView");
    }
    let view = if nullish(event) {
        JsValue::UNDEFINED
    } else {
        get(event, "view")?
    };
    if view.is_truthy() {
        return get(event, "view");
    }
    Ok(fallback.clone().into())
}
