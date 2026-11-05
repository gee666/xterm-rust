// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Read-only diagnostics, not a writable JS option bag. Placeholder inspection
//! reads the same raw value as the public property; other options remain typed.
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
}
impl ImageOptionsReader {
    pub(super) fn new(options: Rc<Cell<ImageOptions>>, placeholder: Rc<RefCell<JsValue>>) -> Self {
        Self {
            options,
            placeholder,
        }
    }
}
#[wasm_bindgen]
impl ImageOptionsReader {
    pub fn get(&self, key: &str) -> JsValue {
        let options = self.options.get();
        match key {
            "enableSizeReports" => options.enable_size_reports.into(),
            "pixelLimit" => options.pixel_limit.into(),
            "sixelSupport" => options.sixel_support.into(),
            "sixelScrolling" => options.sixel_scrolling.into(),
            "sixelPaletteLimit" => options.sixel_palette_limit.into(),
            "sixelSizeLimit" => options.sixel_size_limit.into(),
            "storageLimit" => options.storage_limit.into(),
            "showPlaceholder" => self.placeholder.borrow().clone(),
            "iipSupport" => options.iip_support.into(),
            "iipSizeLimit" => options.iip_size_limit.into(),
            "kittySupport" => options.kitty_support.into(),
            "kittySizeLimit" => options.kitty_size_limit.into(),
            _ => JsValue::UNDEFINED,
        }
    }
}
