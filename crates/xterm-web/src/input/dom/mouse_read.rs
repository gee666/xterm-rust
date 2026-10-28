// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use wasm_bindgen::JsValue;

/// Live reads at MouseService's exact source call sites. Each read is required:
/// an aggregate snapshot would evaluate unrelated getters on skipped paths.
pub trait MouseReadHost {
    fn mouse_require_alt(&self) -> Result<bool, JsValue>;
    fn mouse_has_scrollback(&self) -> Result<bool, JsValue>;
    fn mouse_application_cursor(&self) -> Result<bool, JsValue>;
    fn mouse_cols(&self) -> Result<f64, JsValue>;
    fn mouse_rows(&self) -> Result<f64, JsValue>;
    fn mouse_scroll_sensitivity(&self) -> Result<JsValue, JsValue>;
    fn mouse_fast_scroll_sensitivity(&self) -> Result<JsValue, JsValue>;
    fn mouse_device_cell_height(&self) -> Result<Option<f64>, JsValue>;
    fn mouse_dpr(&self) -> Result<Option<f64>, JsValue>;
    fn mouse_valid_char_size(&self) -> Result<bool, JsValue>;
    fn mouse_canvas_size(&self) -> Result<[f64; 2], JsValue>;
    fn mouse_cell_size(&self) -> Result<[f64; 2], JsValue>;
    fn mouse_css_cell_height(&self) -> Result<Option<f64>, JsValue>;
}
