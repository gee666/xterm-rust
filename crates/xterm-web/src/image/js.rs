// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Checked platform calls preserve thrown JavaScript values for StorageError.
use js_sys::{Array, Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use xterm_addons::image::storage::StorageError;

pub(super) fn error(value: JsValue) -> StorageError {
    StorageError::Event(xterm_wasm_support::js_error::callback_error(value))
}
pub(super) fn get(object: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    Reflect::get(object, &key.into())
}
pub(super) fn set(object: &JsValue, key: &str, value: &JsValue) -> Result<(), JsValue> {
    if Reflect::set(object, &key.into(), value)? {
        Ok(())
    } else {
        Err(js_sys::TypeError::new(&format!("Cannot assign image property {key}")).into())
    }
}
pub(super) fn function(value: JsValue, name: &str) -> Result<Function, JsValue> {
    value
        .dyn_into()
        .map_err(|_| js_sys::TypeError::new(&format!("{name} is not a function")).into())
}
pub(super) fn call(object: &JsValue, key: &str, arguments: &[JsValue]) -> Result<JsValue, JsValue> {
    let method = function(get(object, key)?, key)?;
    let values = arguments.iter().cloned().collect::<Array>();
    method.apply(object, &values)
}
pub(super) fn construct(name: &str, arguments: &[JsValue]) -> Result<JsValue, JsValue> {
    let constructor = function(get(&js_sys::global(), name)?, name)?;
    Reflect::construct(&constructor, &arguments.iter().cloned().collect::<Array>())
}
pub(super) fn nullish(value: &JsValue) -> bool {
    value.is_null() || value.is_undefined()
}
pub(super) fn number_or(value: f64, fallback: f64) -> f64 {
    if value == 0.0 || value.is_nan() {
        fallback
    } else {
        value
    }
}
pub(super) fn number(object: &JsValue, key: &str) -> Result<f64, JsValue> {
    get(object, key)?
        .as_f64()
        .ok_or_else(|| js_sys::TypeError::new(&format!("Image {key} must be a number")).into())
}
pub(super) fn context(canvas: &JsValue, alpha: Option<bool>) -> Result<Option<JsValue>, JsValue> {
    let mut args = vec!["2d".into()];
    if let Some(alpha) = alpha {
        let options = js_sys::Object::new();
        set(&options, "alpha", &alpha.into())?;
        args.push(options.into());
    }
    let result = call(canvas, "getContext", &args)?;
    Ok((!nullish(&result)).then_some(result))
}
