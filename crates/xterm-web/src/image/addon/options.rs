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
// Preserve the operand expression too: some engines include it in TypeErrors.
function kittyEncodedLimit() {
  return Math.ceil(this._opts.kittySizeLimit * 4 / 3);
}
export function imageKittyEncodedLimit(kittySizeLimit) {
  return kittyEncodedLimit.call({_opts: {kittySizeLimit}});
}
// Keep this operand spelling for engine-specific generated TypeError messages.
function iipEncodedLimit() {
  return Math.ceil(this._opts.iipSizeLimit * 4 / 3);
}
export function imageIipEncodedLimit(iipSizeLimit) {
  return iipEncodedLimit.call({_opts: {iipSizeLimit}});
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageAddonOptions)]
    fn clone_options(options: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = imageOptionNumber)]
    pub(in crate::image) fn number(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = imageKittyEncodedLimit)]
    pub(super) fn kitty_encoded_limit(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = imageIipEncodedLimit)]
    pub(super) fn iip_encoded_limit(value: &JsValue) -> Result<f64, JsValue>;
}
pub(super) struct ConstructorOptions {
    pub current: ImageOptions,
    pub defaults: ImageOptions,
    pub placeholder: JsValue,
    pub storage_limit: JsValue,
    pub sixel_size_limit: JsValue,
    pub kitty_size_limit: JsValue,
    pub iip_size_limit: JsValue,
    pub flags: Flags,
    pub default_scrolling: JsValue,
    pub palette: super::palette::Palette,
}

// Truthiness is a native policy projection, not the original option value.
// Keep raw flags separate so a symbol or object never undergoes conversion.
pub(super) struct Flags {
    enable_size_reports: JsValue,
    sixel_support: JsValue,
    pub scrolling: std::cell::RefCell<JsValue>,
    iip_support: JsValue,
    kitty_support: JsValue,
}
impl Flags {
    fn read(options: &JsValue) -> Result<Self, JsValue> {
        Ok(Self {
            enable_size_reports: js::get(options, "enableSizeReports")?,
            sixel_support: js::get(options, "sixelSupport")?,
            scrolling: std::cell::RefCell::new(js::get(options, "sixelScrolling")?),
            iip_support: js::get(options, "iipSupport")?,
            kitty_support: js::get(options, "kittySupport")?,
        })
    }
    pub fn get(&self, key: &str) -> Option<JsValue> {
        Some(match key {
            "enableSizeReports" => self.enable_size_reports.clone(),
            "sixelSupport" => self.sixel_support.clone(),
            "sixelScrolling" => self.scrolling.borrow().clone(),
            "iipSupport" => self.iip_support.clone(),
            "kittySupport" => self.kitty_support.clone(),
            _ => return None,
        })
    }
}
pub(super) fn read(options: &JsValue) -> Result<ConstructorOptions, JsValue> {
    // Keep source's two Object.assign passes contiguous. A failure in the
    // second pass must win over coercion errors in the typed projection.
    let current = clone_options(options)?;
    let defaults = clone_options(options)?;
    Ok(ConstructorOptions {
        placeholder: js::get(&current, "showPlaceholder")?,
        sixel_size_limit: js::get(&current, "sixelSizeLimit")?,
        kitty_size_limit: js::get(&current, "kittySizeLimit")?,
        iip_size_limit: js::get(&current, "iipSizeLimit")?,
        palette: super::palette::Palette::read(&current, &defaults)?,
        current: typed(&current)?,
        defaults: typed(&defaults)?,
        storage_limit: js::get(&current, "storageLimit")?,
        flags: Flags::read(&current)?,
        default_scrolling: js::get(&defaults, "sixelScrolling")?,
    })
}
fn typed(options: &JsValue) -> Result<ImageOptions, JsValue> {
    let boolean = |key| js::get(options, key).map(|v| v.is_truthy());
    let numeric = |key| number(&js::get(options, key)?);
    Ok(ImageOptions {
        enable_size_reports: boolean("enableSizeReports")?,
        pixel_limit: numeric("pixelLimit")?,
        sixel_support: boolean("sixelSupport")?,
        sixel_scrolling: boolean("sixelScrolling")?,
        sixel_palette_limit: ImageOptions::default().sixel_palette_limit,
        // Streaming and storage own delayed raw-value conversions.
        sixel_size_limit: ImageOptions::default().sixel_size_limit,
        storage_limit: ImageOptions::default().storage_limit,
        show_placeholder: boolean("showPlaceholder")?,
        iip_support: boolean("iipSupport")?,
        iip_size_limit: ImageOptions::default().iip_size_limit,
        kitty_support: boolean("kittySupport")?,
        kitty_size_limit: ImageOptions::default().kitty_size_limit,
    })
}
