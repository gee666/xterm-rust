// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::MouseSnapshot;
use wasm_bindgen::JsValue;

/// Fine-grained live reads at MouseService's source call sites. Defaults are for
/// immutable test models only. A DOM terminal overrides these reads so an
/// unrelated option/metric getter cannot run on a short-circuited source path.
pub trait MouseReadHost {
    fn mouse_snapshot(&self) -> Result<MouseSnapshot, JsValue>;

    fn mouse_require_alt(&self) -> Result<bool, JsValue> {
        Ok(self.mouse_snapshot()?.routing.require_alt)
    }
    fn mouse_has_scrollback(&self) -> Result<bool, JsValue> {
        Ok(self.mouse_snapshot()?.has_scrollback)
    }
    fn mouse_application_cursor(&self) -> Result<bool, JsValue> {
        Ok(self.mouse_snapshot()?.application_cursor)
    }
    fn mouse_cols(&self) -> Result<f64, JsValue> {
        Ok(self.mouse_snapshot()?.routing.cols)
    }
    fn mouse_rows(&self) -> Result<f64, JsValue> {
        Ok(self.mouse_snapshot()?.routing.rows)
    }
    fn mouse_scroll_sensitivity(&self) -> Result<f64, JsValue> {
        Ok(self
            .mouse_snapshot()?
            .routing
            .wheel_options
            .scroll_sensitivity)
    }
    fn mouse_fast_scroll_sensitivity(&self) -> Result<f64, JsValue> {
        Ok(self
            .mouse_snapshot()?
            .routing
            .wheel_options
            .fast_scroll_sensitivity)
    }
    fn mouse_device_cell_height(&self) -> Result<Option<f64>, JsValue> {
        Ok(self.mouse_snapshot()?.routing.device_cell_height)
    }
    fn mouse_dpr(&self) -> Result<Option<f64>, JsValue> {
        Ok(self.mouse_snapshot()?.routing.dpr)
    }
    fn mouse_valid_char_size(&self) -> Result<bool, JsValue> {
        Ok(self.mouse_snapshot()?.valid_char_size)
    }
    fn mouse_canvas_size(&self) -> Result<[f64; 2], JsValue> {
        let snapshot = self.mouse_snapshot()?;
        Ok([snapshot.css_canvas_width, snapshot.css_canvas_height])
    }
    fn mouse_cell_size(&self) -> Result<[f64; 2], JsValue> {
        let snapshot = self.mouse_snapshot()?;
        Ok([snapshot.css_cell_width, snapshot.css_cell_height])
    }
    fn mouse_css_cell_height(&self) -> Result<Option<f64>, JsValue> {
        Ok(Some(self.mouse_snapshot()?.css_cell_height))
    }
}
