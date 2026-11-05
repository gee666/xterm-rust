// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::super::js;
use wasm_bindgen::prelude::*;
use xterm_addons::image::options::ImageOptions;

#[wasm_bindgen(inline_js = "
export function imageAddonOptions(options) {
  return Object.assign({}, {
    enableSizeReports: true, pixelLimit: 16777216, sixelSupport: true,
    sixelScrolling: true, sixelPaletteLimit: 4096, sixelSizeLimit: 33554432,
    storageLimit: 128, showPlaceholder: true, iipSupport: true,
    iipSizeLimit: 33554432, kittySupport: true, kittySizeLimit: 33554432
  }, options);
}
export function imageOptionNumber(value) { return +value; }
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageAddonOptions)]
    fn clone_options(options: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = imageOptionNumber)]
    fn number(value: &JsValue) -> Result<f64, JsValue>;
}
pub(super) fn read(options: &JsValue) -> Result<(ImageOptions, ImageOptions), JsValue> {
    // Keep source's two Object.assign passes contiguous. A failure in the
    // second pass must win over coercion errors in the typed projection.
    let current = clone_options(options)?;
    let defaults = clone_options(options)?;
    Ok((typed(&current)?, typed(&defaults)?))
}
fn typed(options: &JsValue) -> Result<ImageOptions, JsValue> {
    let boolean = |key| js::get(options, key).map(|v| v.is_truthy());
    let numeric = |key| number(&js::get(options, key)?);
    Ok(ImageOptions {
        enable_size_reports: boolean("enableSizeReports")?,
        pixel_limit: numeric("pixelLimit")?,
        sixel_support: boolean("sixelSupport")?,
        sixel_scrolling: boolean("sixelScrolling")?,
        sixel_palette_limit: numeric("sixelPaletteLimit")?,
        sixel_size_limit: numeric("sixelSizeLimit")?,
        storage_limit: numeric("storageLimit")?,
        show_placeholder: boolean("showPlaceholder")?,
        iip_support: boolean("iipSupport")?,
        iip_size_limit: numeric("iipSizeLimit")?,
        kitty_support: boolean("kittySupport")?,
        kitty_size_limit: numeric("kittySizeLimit")?,
    })
}
