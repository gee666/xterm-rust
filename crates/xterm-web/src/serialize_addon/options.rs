// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::{js_error::to_js_error, terminal::options::from_js, utility_addons::js_property::get};
use wasm_bindgen::prelude::*;
use xterm_core::options::OptionValue;

pub(super) fn property(value: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    if value.is_undefined() || value.is_null() {
        Ok(JsValue::UNDEFINED)
    } else {
        get(value, key)
    }
}
pub(super) fn index(value: JsValue) -> Result<usize, JsValue> {
    let number = from_js(&value)?
        .to_number()
        .map_err(|error| to_js_error(&error))?;
    // The native serializer uses integral ranges. Reject nonrepresentable input
    // explicitly rather than silently truncating a fractional or negative row.
    // Leave room for the handler's end-exclusive arithmetic on wasm32.
    if !number.is_finite() || number < 0.0 || number.fract() != 0.0 || number >= u32::MAX as f64 {
        return Err(js_sys::RangeError::new(
            "Serialization coordinate is outside the native integral range",
        )
        .into());
    }
    Ok(number as usize)
}
pub(super) fn marker(range: &JsValue, key: &str) -> Result<usize, JsValue> {
    let first = get(range, key)?;
    let second = get(range, key)?;
    index(if first.as_f64().is_some() {
        second
    } else {
        get(&second, "line")?
    })
}
pub(super) fn scrollback_start(max: usize, rows: usize, value: JsValue) -> Result<usize, JsValue> {
    if value.is_undefined() {
        return Ok(0);
    }
    let count = from_js(&value)?
        .js_add(&OptionValue::Number(rows as f64))
        .and_then(|value| value.to_number())
        .map_err(|error| to_js_error(&error))?;
    if count.is_nan() {
        return Err(js_sys::RangeError::new("Invalid array length").into());
    }
    let retained = count.min(max as f64).max(0.0);
    index(JsValue::from_f64(max as f64 - retained))
}
pub(super) fn concat(value: &JsValue) -> Result<Vec<u16>, JsValue> {
    from_js(value)?
        .concat_utf16()
        .map_err(|error| to_js_error(&error))
}
