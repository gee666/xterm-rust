// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Read-only diagnostics, not a writable JS option bag. Placeholder, storage,
//! SIXEL, IIP and Kitty size limits, palette limits and boolean flags retain raw
//! values. Pixel limits remain typed.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use xterm_addons::image::options::ImageOptions;

#[wasm_bindgen(module = "/src/image/addon/option_inspection.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageOptionInspection)]
    pub(super) fn inspection(reader: ImageOptionsReader) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen]
pub struct ImageOptionsReader {
    pub(super) options: Rc<Cell<ImageOptions>>,
    pub(super) placeholder: Rc<RefCell<JsValue>>,
    pub(super) storage_limit: Rc<RefCell<JsValue>>,
    pub(super) sixel_size_limit: JsValue,
    pub(super) kitty_size_limit: JsValue,
    pub(super) iip_size_limit: JsValue,
    pub(super) flags: Rc<super::options::Flags>,
    pub(super) palette: Rc<super::palette::Palette>,
}
#[wasm_bindgen]
impl ImageOptionsReader {
    pub fn get(&self, key: &str) -> JsValue {
        if let Some(value) = self.flags.get(key) {
            return value;
        }
        let options = self.options.get();
        match key {
            "pixelLimit" => options.pixel_limit.into(),
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
