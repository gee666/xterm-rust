// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::{js_error::to_js_error, terminal::options::from_js, utility_addons::js_property::get};
use wasm_bindgen::prelude::*;
use xterm_core::{options::OptionValue, serialize::LineRange};

#[wasm_bindgen(
    inline_js = "export function checkSerializeArrayLength(length) { new Array(length); }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = checkSerializeArrayLength)]
    fn check_array_length(length: f64) -> Result<(), JsValue>;
}

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
pub(super) fn marker(range: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    let first = get(range, key)?;
    let second = get(range, key)?;
    if first.as_f64().is_some() {
        Ok(second)
    } else {
        get(&second, "line")
    }
}
pub(super) fn line_range(start: JsValue, end: JsValue) -> Result<LineRange, JsValue> {
    if let (Some(start), Some(end)) = (start.as_f64(), end.as_f64()) {
        let count = end - start;
        if !count.is_finite() || count < 0.0 || count.fract() != 0.0 || count > u32::MAX as f64 {
            // Source allocates its row array only after reading both markers.
            // Let the browser create its own RangeError, including engine-specific
            // text. The invalid count cannot allocate a large valid array here.
            check_array_length(count)?;
        }
    }
    Ok(LineRange {
        start: index(start)?,
        end: index(end)?,
    })
}
/// Preserve the result of source addition and clamp until the caller selects
/// ANSI's array allocation or HTML's raw numeric row traversal.
pub(super) fn scrollback_row(max: usize, rows: usize, value: JsValue) -> Result<f64, JsValue> {
    if value.is_undefined() {
        return Ok(0.0);
    }
    let count = from_js(&value)?
        .js_add(&OptionValue::Number(rows as f64))
        .and_then(|value| value.to_number())
        .map_err(|error| to_js_error(&error))?;
    if count.is_nan() {
        return Ok(count);
    }
    let retained = count.min(max as f64).max(0.0);
    Ok(max as f64 - retained)
}
pub(super) fn concat(value: &JsValue) -> Result<Vec<u16>, JsValue> {
    from_js(value)?
        .concat_utf16()
        .map_err(|error| to_js_error(&error))
}
