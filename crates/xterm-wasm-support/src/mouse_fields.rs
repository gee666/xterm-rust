// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Raw host fields use the same Rust protocol policies as native events.
use crate::utf16::{from_js_string, to_js_string};
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::mouse_fields::{Encoding, Field, Modifier, MouseFields, Protocol};

// Only language operators cross this boundary. In particular neither Number()
// nor String() substitutes for the source operators' BigInt/Symbol semantics.
#[wasm_bindgen(inline_js = r#"
export function mouseGet(v, k) { return v[k]; }
export function mouseSet(v, k, x) { v[k] = x; }
export function mouseAnd(v, n) { return v & n; }
export function mouseOr(n, v) { return n | v; }
export function mouseAdd(v, n) { return v + n; }
export function mouseGreater(v, n) { return v > n; }
export function mouseCharCode(v) { return v & 65535; }
export function mouseText(v) { return `${v}`; }
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = mouseGet)]
    fn get(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = mouseSet)]
    fn set(value: &JsValue, key: &str, field: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = mouseAnd)]
    fn bit_and(value: &JsValue, mask: i32) -> Result<i32, JsValue>;
    #[wasm_bindgen(catch, js_name = mouseOr)]
    fn bit_or(code: i32, value: &JsValue) -> Result<i32, JsValue>;
    #[wasm_bindgen(catch, js_name = mouseAdd)]
    fn add(value: &JsValue, number: i32) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = mouseGreater)]
    fn greater(value: &JsValue, number: i32) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = mouseCharCode)]
    fn char_code(value: &JsValue) -> Result<u16, JsValue>;
    #[wasm_bindgen(catch, js_name = mouseText)]
    fn text(value: &JsValue) -> Result<js_sys::JsString, JsValue>;
}

pub struct RawMouseEvent(pub JsValue);

impl MouseFields for RawMouseEvent {
    type Value = JsValue;
    type Error = JsValue;

    fn get(&self, field: Field) -> Result<JsValue, JsValue> {
        get(&self.0, field.name())
    }
    fn set_false(&mut self, modifier: Modifier) -> Result<(), JsValue> {
        set(&self.0, modifier.field().name(), &JsValue::FALSE)
    }
    fn truth(value: &JsValue) -> bool {
        value.is_truthy()
    }
    fn equals_number(value: &JsValue, number: i32) -> bool {
        value.as_f64() == Some(f64::from(number))
    }
    fn bit_and(value: &JsValue, mask: i32) -> Result<i32, JsValue> {
        bit_and(value, mask)
    }
    fn bit_or(code: i32, value: &JsValue) -> Result<i32, JsValue> {
        bit_or(code, value)
    }
    fn add_number(value: &JsValue, number: i32) -> Result<JsValue, JsValue> {
        add(value, number)
    }
    fn greater(value: &JsValue, number: i32) -> Result<bool, JsValue> {
        greater(value, number)
    }
    fn char_code(value: &JsValue) -> Result<u16, JsValue> {
        char_code(value)
    }
    fn text(value: &JsValue) -> Result<Vec<u16>, JsValue> {
        Ok(from_js_string(&text(value)?))
    }
}

/// These functions are retained by the ordinary JS registry. They own no event
/// or service state and are released by wasm-bindgen's JS closure finalizer.
pub fn restriction(policy: Protocol) -> js_sys::Function {
    Closure::<dyn Fn(JsValue) -> Result<bool, JsValue>>::new(move |event| {
        policy.restrict(&mut RawMouseEvent(event))
    })
    .into_js_value()
    .unchecked_into()
}

pub fn encoder(policy: Encoding) -> js_sys::Function {
    Closure::<dyn Fn(JsValue) -> Result<JsValue, JsValue>>::new(move |event| {
        Ok(to_js_string(&policy.encode(&RawMouseEvent(event))?).into())
    })
    .into_js_value()
    .unchecked_into()
}
