// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Live structural reads. Native Touch numeric fields stay JS numbers, while
//! property and method getter failures retain their original thrown value.
use super::TouchId;
use wasm_bindgen::{prelude::wasm_bindgen, JsCast, JsValue};

#[wasm_bindgen(
    inline_js = "export function touchField(target, key) { return target[key]; }\nexport function missingTouchRecord(timestamp) { let data; return timestamp ? data.initialTimeStamp : data.initialTarget; }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = touchField)]
    fn read_touch_field(target: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = missingTouchRecord)]
    fn throw_missing_record(timestamp: bool) -> Result<JsValue, JsValue>;
}
pub(super) fn missing_record(timestamp: bool) -> JsValue {
    // Both fixed branches access a literal uninitialized local. ECMAScript must
    // throw, generating this browser's native TypeError text rather than V8 text.
    throw_missing_record(timestamp).expect_err("uninitialized record access always throws")
}

pub(super) fn warn(text: &str, touch: &JsValue) -> Result<(), JsValue> {
    let console = field(&js_sys::global(), "console")?;
    function(&console, "warn")?.call2(&console, &text.into(), touch)?;
    Ok(())
}

pub(super) fn now() -> Result<f64, JsValue> {
    let date = field(&js_sys::global(), "Date")?;
    value_number(function(&date, "now")?.call0(&date)?, "clock")
}
pub(super) fn tap_time() -> Result<f64, JsValue> {
    let date = function(&js_sys::global(), "Date")?;
    let instance = js_sys::Reflect::construct(&date, &js_sys::Array::new())?;
    value_number(function(&instance, "getTime")?.call0(&instance)?, "clock")
}
fn value_number(value: JsValue, name: &str) -> Result<f64, JsValue> {
    value
        .as_f64()
        .ok_or_else(|| js_sys::TypeError::new(&format!("Touch {name} is not a number")).into())
}
pub(super) fn field(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    // Direct GetV retains primitive boxing and this browser's native exceptions.
    read_touch_field(target, name)
}
pub(super) fn number(target: &JsValue, name: &str) -> Result<f64, JsValue> {
    value_number(field(target, name)?, name)
}
pub(super) fn identifier(touch: &JsValue) -> Result<TouchId, JsValue> {
    let value = number(touch, "identifier")?;
    // Number property keys stringify NaNs identically and both signed zeros as
    // "0". Every other Number's shortest string uniquely identifies its value.
    Ok(TouchId(if value == 0.0 {
        0
    } else if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }))
}
pub(super) fn length(event: &JsValue, name: &str) -> Result<f64, JsValue> {
    number(&field(event, name)?, "length")
}
pub(super) fn call0(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    function(target, name)?.call0(target)
}
pub(super) fn function(target: &JsValue, name: &str) -> Result<js_sys::Function, JsValue> {
    field(target, name)?
        .dyn_into()
        .map_err(|_| js_sys::TypeError::new(&format!("{name} is not a function")).into())
}
pub(super) fn item(event: &JsValue, name: &str, index: f64) -> Result<JsValue, JsValue> {
    let list = field(event, name)?;
    let callback = function(&list, "item")?;
    callback.call1(&list, &index.into())
}
