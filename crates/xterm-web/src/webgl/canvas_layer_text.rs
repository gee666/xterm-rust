// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Base canvas font construction and persistent cell clipping.
use super::{args, call, rect, set, CanvasLayer};
use js_sys::{Array, JsString};
use wasm_bindgen::JsValue;
use xterm_core::buffer::CellData;

impl CanvasLayer {
    pub fn font(&self, bold: bool, italic: bool) -> Result<JsValue, JsValue> {
        let options = self.services.terminal.options();
        let get = |name| {
            options
                .get(name)
                .map_err(|e| crate::js_error::to_js_error(&e))
        };
        let weight = get(if bold { "fontWeightBold" } else { "fontWeight" })?
            .to_utf16()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let size = get("fontSize")?
            .to_number()
            .map_err(|e| crate::js_error::to_js_error(&e))?
            * self.services.browser.dpr()?;
        let family = get("fontFamily")?
            .to_utf16()
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        let mut units: Vec<_> = if italic { "italic " } else { " " }
            .encode_utf16()
            .collect();
        units.extend(weight);
        units.extend(format!(" {}px ", js_sys::Number::from(size).to_string(10)?).encode_utf16());
        units.extend(family);
        Ok(JsString::from_char_code(&units).into())
    }
    /// The source does not save/restore the clip. Callers own that stack.
    pub fn fill_char_true_color(&self, cell: &CellData, x: f64, y: f64) -> Result<(), JsValue> {
        let context = self.context_value();
        set(&context, "font", &self.font(false, false)?)?;
        set(
            &self.context_value(),
            "textBaseline",
            &crate::webgl::atlas_context::text_baseline(),
        )?;
        call(&self.context_value(), "beginPath", &Array::new())?;
        let dim = self.dimensions.get().device;
        rect(
            &self.context_value(),
            "rect",
            x * dim.cell.width,
            y * dim.cell.height,
            f64::from(cell.get_width()) * dim.cell.width,
            dim.cell.height,
        )?;
        call(&self.context_value(), "clip", &Array::new())?;
        let dim = self.dimensions.get().device;
        call(
            &self.context_value(),
            "fillText",
            &args(&[
                JsString::from_char_code(&cell.get_chars()).into(),
                (x * dim.cell.width + dim.character.left).into(),
                (y * dim.cell.height + dim.character.top + dim.character.height).into(),
            ]),
        )?;
        Ok(())
    }
}
