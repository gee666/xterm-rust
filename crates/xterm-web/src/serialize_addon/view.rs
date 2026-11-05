// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::js_error::{to_js_error, JsCallbackError};
use std::{error::Error, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::{
    buffer::{BufferError, CellData},
    event::EmitError,
    public_api::BufferApiView,
    serialize::{BufferAccess, PublicBufferView, SerializeError},
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
pub(super) struct BrowserView(PublicBufferView);
impl BrowserView {
    pub(super) fn new(view: Rc<BufferApiView>) -> Result<Self, JsValue> {
        PublicBufferView::new(view)
            .map(Self)
            .map_err(|cause| error(&cause))
    }
}
impl BufferAccess for BrowserView {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        self.0.line_len(row)
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        self.0.cell(row, col)
    }
    fn is_wrapped(&self, row: usize) -> Result<bool, SerializeError> {
        self.0.is_wrapped(row)
    }
    fn cursor_x(&self) -> usize {
        self.0.cursor_x()
    }
    fn cursor_y(&self) -> usize {
        self.0.cursor_y()
    }
    fn base_y(&self) -> usize {
        self.0.base_y()
    }
    fn missing_cell(&self, row: usize, col: usize) -> Result<(), SerializeError> {
        warn(row, col).map_err(|cause| {
            BufferError::Dispatch(EmitError::callback(JsCallbackError(cause))).into()
        })
    }
}
