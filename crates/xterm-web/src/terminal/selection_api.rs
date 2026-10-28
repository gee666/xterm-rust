// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Public selection methods use the same service as mouse input and search.
use super::Terminal;
use crate::{
    api::verify_integers,
    js_error::to_js_error,
    selection::{model::Position, service::buffer_dimensions},
    utf16::to_js_string,
};
use js_sys::{JsString, Object, Reflect};
use std::rc::Rc;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(js_name = hasSelection)]
    pub fn has_selection(&self) -> Result<bool, JsValue> {
        if self.shared.dom.borrow().is_none() {
            return Ok(false);
        }
        let buffers = self
            .shared
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))
            .map_err(|error| to_js_error(&error))?;
        self.shared
            .selection
            .has_selection(buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?)
            .map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(js_name = getSelection)]
    pub fn get_selection(&self) -> Result<JsString, JsValue> {
        if self.shared.dom.borrow().is_none() {
            return Ok(to_js_string(&[]));
        }
        let buffers = self
            .shared
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))
            .map_err(|error| to_js_error(&error))?;
        let active = buffers.buffer();
        let active = active.try_borrow().map_err(|error| to_js_error(&error))?;
        let text = self
            .shared
            .selection
            .selection_text(
                &active,
                buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?,
            )
            .map_err(|error| to_js_error(&error))?;
        Ok(to_js_string(&text))
    }
    #[wasm_bindgen(js_name = getSelectionPosition)]
    pub fn get_selection_position(&self) -> Result<JsValue, JsValue> {
        if !self.has_selection()? {
            return Ok(JsValue::UNDEFINED);
        }
        let buffers = self
            .shared
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))
            .map_err(|error| to_js_error(&error))?;
        let start = self.shared.selection.selection_start();
        let end = self
            .shared
            .selection
            .selection_end(buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?)
            .map_err(|error| to_js_error(&error))?;
        let (Some(start), Some(end)) = (start, end) else {
            return Ok(JsValue::UNDEFINED);
        };
        let object = Object::new();
        set(&object, "start", &position(start)?)?;
        set(&object, "end", &position(end)?)?;
        Ok(object.into())
    }
    pub fn select(&self, column: f64, row: f64, length: f64) -> Result<(), JsValue> {
        verify_integers(&[column, row, length]).map_err(|error| to_js_error(&error))?;
        // CoreBrowserTerminal.select dereferences its service before open,
        // unlike the optional chaining used by clear/selectAll/selectLines.
        if self.shared.dom.borrow().is_none() {
            return Err(js_sys::TypeError::new(
                "Cannot read properties of undefined (reading 'setSelection')",
            )
            .into());
        }
        let buffers = self
            .shared
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))
            .map_err(|error| to_js_error(&error))?;
        self.shared
            .selection
            .set_selection(
                &buffers,
                buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?,
                Position::new(coordinate(column)?, coordinate(row)?),
                coordinate(length)?,
            )
            .map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(js_name = clearSelection)]
    pub fn clear_selection(&self) -> Result<(), JsValue> {
        // This before-open no-op is source behavior, not an absent implementation.
        if self.shared.dom.borrow().is_none() {
            return Ok(());
        }
        let buffers = self
            .shared
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))
            .map_err(|error| to_js_error(&error))?;
        self.shared
            .selection
            .clear_selection(
                &buffers,
                buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?,
            )
            .map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(js_name = selectAll)]
    pub fn select_all(&self) -> Result<(), JsValue> {
        if self.shared.dom.borrow().is_none() {
            return Ok(());
        }
        let buffers = self
            .shared
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))
            .map_err(|error| to_js_error(&error))?;
        self.shared
            .selection
            .select_all(
                &buffers,
                buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?,
            )
            .map_err(|error| to_js_error(&error))
    }
    #[wasm_bindgen(js_name = selectLines)]
    pub fn select_lines(&self, start: f64, end: f64) -> Result<(), JsValue> {
        verify_integers(&[start, end]).map_err(|error| to_js_error(&error))?;
        if self.shared.dom.borrow().is_none() {
            return Ok(());
        }
        let buffers = self
            .shared
            .native
            .with_state(|state| Rc::clone(&state.buffer_service))
            .map_err(|error| to_js_error(&error))?;
        self.shared
            .selection
            .select_lines(
                &buffers,
                buffer_dimensions(&buffers).map_err(|error| to_js_error(&error))?,
                coordinate(start)?,
                coordinate(end)?,
            )
            .map_err(|error| to_js_error(&error))
    }
}
fn position(value: Position) -> Result<JsValue, JsValue> {
    let object = Object::new();
    set(&object, "x", &(value.column as f64).into())?;
    set(&object, "y", &(value.row as f64).into())?;
    Ok(object.into())
}
fn set(object: &Object, key: &str, value: &JsValue) -> Result<(), JsValue> {
    if Reflect::set(object, &key.into(), value)? {
        Ok(())
    } else {
        Err(js_sys::TypeError::new("Selection position field is not writable").into())
    }
}
fn coordinate(value: f64) -> Result<i64, JsValue> {
    // Criterion420 tracks source Number values outside this native representation
    // and negative zero. Never silently saturate a large validated JS integer.
    const I64_EXCLUSIVE_MAX: f64 = 9_223_372_036_854_775_808.0;
    if value < i64::MIN as f64 || value >= I64_EXCLUSIVE_MAX {
        return Err(js_sys::RangeError::new(
            "Selection coordinate exceeds the Rust signed 64-bit representation",
        )
        .into());
    }
    Ok(value as i64)
}
