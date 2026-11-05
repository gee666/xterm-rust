// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Read-only JavaScript snapshot of actual Rust packed row storage.
use super::WebBufferCell;
use crate::{js_error::to_js_error, utf16::to_js_string};
use js_sys::{Object, Reflect, Uint32Array};
use wasm_bindgen::JsValue;
use xterm_core::buffer::LineRef;

pub(super) fn snapshot(line: &LineRef) -> Result<JsValue, JsValue> {
    let (words, combined, extended, length, wrapped) = {
        let line = line.try_borrow().map_err(|error| to_js_error(&error))?;
        (
            line.packed_cells()
                .iter()
                .flatten()
                .copied()
                .collect::<Vec<_>>(),
            line.combined_strings().clone(),
            line.extended_attributes()
                .iter()
                .map(|(&index, value)| (index, value.share()))
                .collect::<Vec<_>>(),
            line.len(),
            line.is_wrapped,
        )
    };
    // Creating JS fields can reenter through host hooks. No native row borrow
    // survives the capture above; payload objects retain their actual identity.
    let object = Object::new();
    set(&object, "_data", &Uint32Array::from(words.as_slice()))?;
    let strings = Object::new();
    for (index, value) in combined {
        set(&strings, &index.to_string(), &to_js_string(&value))?;
    }
    set(&object, "_combined", &strings)?;
    let attrs = Object::new();
    for (index, value) in extended {
        set(
            &attrs,
            &index.to_string(),
            &WebBufferCell::extended_value(Some(value))?,
        )?;
    }
    set(&object, "_extendedAttrs", &attrs)?;
    set(&object, "length", &JsValue::from_f64(length as f64))?;
    set(&object, "_isWrapped", &JsValue::from_bool(wrapped))?;
    Ok(object.into())
}
fn set(object: &Object, key: &str, value: &JsValue) -> Result<(), JsValue> {
    if Reflect::set(object, &key.into(), value)? {
        Ok(())
    } else {
        Err(js_sys::TypeError::new("Buffer snapshot field is not writable").into())
    }
}
