// Copyright (c) 2020 The xterm.js authors. MIT license.
//! Geometry requests evaluate the raw limit once per source expression.
use super::{
    super::{pixel_limit, terminal_host::TerminalCanvasHost, ImageCanvasHost},
    State,
};
use crate::js_error::to_js_error;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use xterm_addons::image::{
    policy::{ImageControlAction, ImageControlEffect, ImageParameter},
    storage::CELL_SIZE_DEFAULT,
};
const SIXEL_GEOMETRY: f64 = 2.0;
const READ: f64 = 1.0;
const READ_MAX: f64 = 4.0;
impl State {
    pub(super) fn pixel_control(&self, params: &[ImageParameter]) -> Option<Result<bool, JsValue>> {
        if params.len() < 2 || params[0] != ImageParameter::Number(SIXEL_GEOMETRY) {
            return None;
        }
        match params[1] {
            ImageParameter::Number(READ) => Some(self.geometry_reply()),
            ImageParameter::Number(READ_MAX) => Some(self.square_reply()),
            _ => None,
        }
    }
    fn geometry_reply(&self) -> Result<bool, JsValue> {
        let terminal = self.terminal.borrow().upgrade();
        let mut columns = 80.0;
        let mut rows = 24.0;
        let mut width = 0.0;
        let mut height = 0.0;
        if let Some(terminal) = terminal {
            if let Some(dimensions) = TerminalCanvasHost(Rc::downgrade(&terminal)).dimensions()? {
                width = dimensions.canvas_width;
                height = dimensions.canvas_height;
            }
            // Fallback terminal geometry is read only when canvas dimensions
            // are falsey, before evaluating the limit's comparison.
            if !truthy(width) || !truthy(height) {
                columns = terminal.native.cols().map_err(|e| to_js_error(&e))? as f64;
                rows = terminal.native.rows().map_err(|e| to_js_error(&e))? as f64;
            }
        }
        if !truthy(width) || !truthy(height) {
            width = if truthy(columns) { columns } else { 80.0 } * CELL_SIZE_DEFAULT.width;
            height = if truthy(rows) { rows } else { 24.0 } * CELL_SIZE_DEFAULT.height;
        }
        if pixel_limit::below(width, height, &self.pixel_limit)? {
            self.report_geometry(fixed_reply(width, height)?)
        } else {
            // The square expression converts again. A stateful object can
            // return a different value or throw after the comparison.
            self.square_reply()
        }
    }
    fn square_reply(&self) -> Result<bool, JsValue> {
        let square = pixel_limit::square(&self.pixel_limit)?;
        self.report_geometry(square_reply(square)?)
    }
    fn report_geometry(&self, reply: js_sys::JsString) -> Result<bool, JsValue> {
        self.execute(ImageControlAction {
            handled: true,
            effects: vec![ImageControlEffect::Reply(crate::utf16::from_js_string(
                &reply,
            ))],
        })
    }
}
fn truthy(value: f64) -> bool {
    value != 0.0 && !value.is_nan()
}
#[wasm_bindgen(inline_js = "
export function imageFixedGeometryReply(width, height) {
  return '\\x1b[?2;0;' + width.toFixed(0) + ';' + height.toFixed(0) + 'S';
}
export function imageSquareGeometryReply(x) {
  return '\\x1b[?2;0;' + x + ';' + x + 'S';
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageFixedGeometryReply)]
    fn fixed_reply(width: f64, height: f64) -> Result<js_sys::JsString, JsValue>;
    #[wasm_bindgen(catch, js_name = imageSquareGeometryReply)]
    fn square_reply(value: f64) -> Result<js_sys::JsString, JsValue>;
}
