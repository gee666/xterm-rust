// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Read-only JavaScript captures of actual Rust packed row storage.
use super::WebBufferCell;
use crate::{js_error::to_js_error, utf16::to_js_string};
use js_sys::{Object, Reflect, Uint32Array};
use wasm_bindgen::JsValue;
use xterm_core::buffer::LineRef;

pub(super) fn data(line: &LineRef) -> Result<Uint32Array, JsValue> {
    let words = line
        .try_borrow()
        .map_err(|error| to_js_error(&error))?
        .packed_cells()
        .iter()
        .flatten()
        .copied()
        .collect::<Vec<_>>();
    Ok(Uint32Array::from(words.as_slice()))
}
pub(super) fn combined(line: &LineRef) -> Result<Object, JsValue> {
    let combined = line
        .try_borrow()
        .map_err(|error| to_js_error(&error))?
        .combined_strings()
        .clone();
    let object = Object::new();
    for (index, value) in combined {
        set(&object, &index.to_string(), &to_js_string(&value))?;
    }
    Ok(object)
}
pub(super) fn extended(line: &LineRef) -> Result<Object, JsValue> {
    let extended = line
        .try_borrow()
        .map_err(|error| to_js_error(&error))?
        .extended_attributes()
        .iter()
        .map(|(&index, value)| (index, value.share()))
        .collect::<Vec<_>>();
    // No native row borrow survives JS field creation or host payload access.
    let object = Object::new();
    for (index, value) in extended {
        set(
            &object,
            &index.to_string(),
            &WebBufferCell::extended_value(Some(value))?,
        )?;
    }
    Ok(object)
}
fn set(object: &Object, key: &str, value: &JsValue) -> Result<(), JsValue> {
    if Reflect::set(object, &key.into(), value)? {
        Ok(())
    } else {
        Err(js_sys::TypeError::new("Buffer snapshot field is not writable").into())
    }
}
