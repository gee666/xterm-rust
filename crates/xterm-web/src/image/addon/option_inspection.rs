// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Read-only diagnostics, not a writable JS option bag. Placeholder, storage,
//! SIXEL, IIP and Kitty size limits and boolean flags retain raw values.
//! Pixel and palette limits remain typed.
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
    options: Rc<Cell<ImageOptions>>,
    placeholder: Rc<RefCell<JsValue>>,
    storage_limit: Rc<RefCell<JsValue>>,
    sixel_size_limit: JsValue,
    kitty_size_limit: JsValue,
    iip_size_limit: JsValue,
    flags: Rc<super::options::Flags>,
}
impl ImageOptionsReader {
    pub(super) fn new(
        options: Rc<Cell<ImageOptions>>,
        placeholder: Rc<RefCell<JsValue>>,
        storage_limit: Rc<RefCell<JsValue>>,
        sixel_size_limit: JsValue,
        kitty_size_limit: JsValue,
        iip_size_limit: JsValue,
        flags: Rc<super::options::Flags>,
    ) -> Self {
        Self {
            options,
            placeholder,
            storage_limit,
            sixel_size_limit,
            kitty_size_limit,
            iip_size_limit,
            flags,
        }
    }
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
            "sixelPaletteLimit" => options.sixel_palette_limit.into(),
            "sixelSizeLimit" => self.sixel_size_limit.clone(),
            "storageLimit" => self.storage_limit.borrow().clone(),
            "showPlaceholder" => self.placeholder.borrow().clone(),
            "iipSizeLimit" => self.iip_size_limit.clone(),
            "kittySizeLimit" => self.kitty_size_limit.clone(),
            _ => JsValue::UNDEFINED,
        }
    }
}
