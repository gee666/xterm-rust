// Copyright (c) 2019, 2021 The xterm.js authors. MIT license.
//! Internal width entry point used by upstream browser addon suites.

use crate::{js_error::to_js_error, utf16::from_js_string};
use js_sys::JsString;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use xterm_core::unicode_service::UnicodeService;

/// A retained view of the terminal's actual service, not a separate registry.
/// Only the width entry point is exported here; the public Unicode API owns
/// registration and version selection.
#[wasm_bindgen(js_name = UnicodeService)]
pub struct WebUnicodeService {
    service: Rc<UnicodeService>,
}

impl WebUnicodeService {
    pub fn new(service: Rc<UnicodeService>) -> Self {
        Self { service }
    }
}

#[wasm_bindgen(js_class = UnicodeService)]
impl WebUnicodeService {
    #[wasm_bindgen(js_name = getStringCellWidth)]
    pub fn get_string_cell_width(&self, text: JsString) -> Result<f64, JsValue> {
        self.service
            .get_string_cell_width_utf16(&from_js_string(&text))
            // JS string lengths fit u32 and a unit contributes at most three
            // columns, so this integer-to-Number conversion is exact.
            .map(|width| width as f64)
            .map_err(|error| to_js_error(&error))
    }
}
