// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Read-only diagnostics of typed native options, not a writable JS option bag.
use std::{cell::Cell, rc::Rc};
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
}
impl ImageOptionsReader {
    pub(super) fn new(options: Rc<Cell<ImageOptions>>) -> Self {
        Self { options }
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
            "showPlaceholder" => options.show_placeholder.into(),
            "iipSupport" => options.iip_support.into(),
            "iipSizeLimit" => options.iip_size_limit.into(),
            "kittySupport" => options.kitty_support.into(),
            "kittySizeLimit" => options.kitty_size_limit.into(),
            _ => JsValue::UNDEFINED,
        }
    }
}
