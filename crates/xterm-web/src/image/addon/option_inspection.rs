// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Read-only diagnostics, not a writable JS option bag. Placeholder, storage,
//! SIXEL, IIP and Kitty size limits, pixel/palette limits and boolean flags retain
//! raw values.
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/image/addon/option_inspection.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageOptionInspection)]
    pub(super) fn inspection(reader: ImageOptionsReader) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
pub struct ImageOptionsReader {
    pub(super) placeholder: Rc<RefCell<JsValue>>,
    pub(super) storage_limit: Rc<RefCell<JsValue>>,
    pub(super) sixel_size_limit: JsValue,
    pub(super) kitty_size_limit: JsValue,
    pub(super) iip_size_limit: JsValue,
    pub(super) flags: Rc<super::options::Flags>,
    pub(super) palette: Rc<super::palette::Palette>,
    pub(super) pixel_limit: JsValue,
}
#[wasm_bindgen]
impl ImageOptionsReader {
    pub fn get(&self, key: &str) -> JsValue {
        if let Some(value) = self.flags.get(key) {
            return value;
        }
        match key {
            "pixelLimit" => self.pixel_limit.clone(),
            "sixelPaletteLimit" => self.palette.get(),
            "sixelSizeLimit" => self.sixel_size_limit.clone(),
            "storageLimit" => self.storage_limit.borrow().clone(),
            "showPlaceholder" => self.placeholder.borrow().clone(),
            "iipSizeLimit" => self.iip_size_limit.clone(),
            "kittySizeLimit" => self.kitty_size_limit.clone(),
            _ => JsValue::UNDEFINED,
        }
    }
}
