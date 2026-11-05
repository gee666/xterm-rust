// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{bindings::surface, State};
use crate::js_error::to_js_error;
use wasm_bindgen::prelude::*;
use xterm_core::buffer::constants::{bg_flags, CELL_BG_WORD, CELL_WORDS};

#[wasm_bindgen(module = "/src/image/addon/coordinates.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageRowSum)]
    fn row_sum(start: f64, index: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = imageCellWord)]
    fn cell_word(index: &JsValue, words: f64) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = imageCellProperty)]
    fn cell_property(index: &JsValue) -> Result<JsValue, JsValue>;
}

pub(super) enum CellLookup {
    Image,
    Tile,
}

impl State {
    pub(super) fn lookup_cell(
        &self,
        x: &JsValue,
        y: &JsValue,
        kind: CellLookup,
    ) -> Result<JsValue, JsValue> {
        let Some(storage) = self.storage() else {
            return Ok(JsValue::UNDEFINED);
        };
        // Capture receiver, active buffer and backing before y conversion, then
        // line before x multiplication and property map before key conversion.
        // Neither native buffer nor map borrows cross JavaScript user hooks.
        let read = storage
            .prepare_buffer_line_read()
            .map_err(|e| to_js_error(&e))?;
        let Some(line) = read.get_with(|start| row_sum(start, y))? else {
            return Ok(JsValue::UNDEFINED);
        };
        let address = cell_word(x, CELL_WORDS as f64)? + CELL_BG_WORD as f64;
        let properties = {
            let line = line.try_borrow().map_err(|e| to_js_error(&e))?;
            if line.numeric_word(address).unwrap_or(0) & bg_flags::HAS_EXTENDED == 0 {
                return Ok(JsValue::UNDEFINED);
            }
            line.prepare_extended_read()
        };
        let Some(key) = property_index(x)? else {
            return Ok(JsValue::UNDEFINED);
        };
        let Some(extended) = properties.get(key) else {
            return Ok(JsValue::UNDEFINED);
        };
        let resource = match kind {
            CellLookup::Image => storage.get_image_from_extended(&extended),
            CellLookup::Tile => storage.extract_tile_from_extended(&extended),
        }
        .map_err(|e| to_js_error(&e))?;
        surface(resource)
    }
}

fn property_index(value: &JsValue) -> Result<Option<usize>, JsValue> {
    if let Some(number) = value.as_f64() {
        return Ok(
            if number.is_finite()
                && number >= 0.0
                && number.fract() == 0.0
                && number < usize::MAX as f64
            {
                Some(number as usize)
            } else {
                None
            },
        );
    }
    let key = if value.is_object() || value.is_function() {
        cell_property(value)?
    } else {
        value.clone()
    };
    // Only canonical integer keys can contain addon-created payloads. Symbols,
    // other primitives and noncanonical numeric strings have no image assigned.
    Ok(key.as_string().and_then(|key| {
        key.parse::<usize>()
            .ok()
            .filter(|index| index.to_string() == key)
    }))
}
