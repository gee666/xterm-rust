// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::{
    js_error::{to_js_error, JsCallbackError},
    terminal::Shared,
};
use std::{error::Error, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::{
    buffer::{AttributeData, BufferError, CellData},
    event::EmitError,
    public_api::BufferApiView,
    serialize::{BufferAccess, Position, PublicBufferView, SerializeError},
};

#[wasm_bindgen(
    inline_js = "export function serializeMissingCell(row,col) { console.warn(`Can't get cell at row=${row}, col=${col}`); }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = serializeMissingCell)]
    fn warn(row: usize, col: usize) -> Result<(), JsValue>;
}
pub(super) fn error(error: &(dyn Error + 'static)) -> JsValue {
    let mut current = Some(error);
    while let Some(cause) = current {
        if let Some(cause) = cause.downcast_ref::<SerializeError>() {
            match cause {
                SerializeError::InvalidRange => {
                    return js_sys::RangeError::new(&cause.to_string()).into()
                }
                SerializeError::MissingWrappedLine => {
                    return js_sys::TypeError::new(&cause.to_string()).into()
                }
                _ => {}
            }
        }
        current = cause.source();
    }
    to_js_error(error)
}
fn native_error(cause: impl Error + 'static) -> SerializeError {
    BufferError::Dispatch(EmitError::callback(JsCallbackError(to_js_error(&cause)))).into()
}
pub(super) struct BrowserView<'a> {
    captured: PublicBufferView,
    view: Rc<BufferApiView>,
    host: &'a Shared,
}
impl<'a> BrowserView<'a> {
    pub(super) fn new(view: Rc<BufferApiView>, host: &'a Shared) -> Result<Self, JsValue> {
        let captured = PublicBufferView::new(view.clone()).map_err(|cause| error(&cause))?;
        Ok(Self {
            captured,
            view,
            host,
        })
    }
}
impl BufferAccess for BrowserView<'_> {
    fn len(&self) -> usize {
        self.captured.len()
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        self.captured.line_len(row)
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        self.captured.cell(row, col)
    }
    fn is_wrapped(&self, row: usize) -> Result<bool, SerializeError> {
        self.captured.is_wrapped(row)
    }
    fn cursor_x(&self) -> usize {
        self.captured.cursor_x()
    }
    fn cursor_y(&self) -> usize {
        self.captured.cursor_y()
    }
    fn base_y(&self) -> usize {
        self.captured.base_y()
    }
    fn current_rows(&self, _: usize) -> Result<usize, SerializeError> {
        self.host.native.rows().map_err(native_error)
    }
    fn current_length(&self) -> Result<usize, SerializeError> {
        self.view.len().map_err(Into::into)
    }
    fn current_cursor(&self) -> Result<Position, SerializeError> {
        Ok(Position {
            y: self.view.base_y()? + self.view.cursor_y()?,
            x: self.view.cursor_x()?,
        })
    }
    fn current_attrs(&self, _: &AttributeData) -> Result<AttributeData, SerializeError> {
        self.host
            .native
            .with_state(|state| state.attributes.clone())
            .map_err(native_error)
    }
    fn missing_cell(&self, row: usize, col: usize) -> Result<(), SerializeError> {
        warn(row, col).map_err(|cause| {
            BufferError::Dispatch(EmitError::callback(JsCallbackError(cause))).into()
        })
    }
}
