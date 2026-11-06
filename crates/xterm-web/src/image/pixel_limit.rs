// Copyright (c) 2020, 2023, 2026 The xterm.js authors. MIT license.
//! Raw JS arithmetic belongs at consumption sites, not constructor projection.
//! These helpers retain relational BigInt semantics and exact host exceptions.
use wasm_bindgen::prelude::*;
#[wasm_bindgen(inline_js = "
function sixelMemoryLimit() { return this._opts.pixelLimit * 4; }
function belowPixelLimit(w, h) { return w * h < this._opts.pixelLimit; }
function abovePixelLimit(w, h) { return w * h > this._opts.pixelLimit; }
function squaredPixelLimit() { return Math.floor(Math.sqrt(this._opts.pixelLimit)); }
export function imageSixelMemoryLimit(pixelLimit) {
  return sixelMemoryLimit.call({_opts: {pixelLimit}});
}
export function imageBelowPixelLimit(w, h, pixelLimit) {
  return belowPixelLimit.call({_opts: {pixelLimit}}, w, h);
}
export function imageAbovePixelLimit(w, h, pixelLimit) {
  return abovePixelLimit.call({_opts: {pixelLimit}}, w, h);
}
export function imageSquaredPixelLimit(pixelLimit) {
  return squaredPixelLimit.call({_opts: {pixelLimit}});
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageSixelMemoryLimit)]
    pub(super) fn memory(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = imageBelowPixelLimit)]
    pub(super) fn below(width: f64, height: f64, value: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = imageAbovePixelLimit)]
    pub(super) fn above(width: f64, height: f64, value: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = imageSquaredPixelLimit)]
    pub(super) fn square(value: &JsValue) -> Result<f64, JsValue>;
}
/// A producer can also be constructed from native numeric options. The addon
/// passes its retained raw value instead, without converting it in advance.
#[derive(Clone)]
pub(super) struct PixelLimit(JsValue);
impl PixelLimit {
    pub(super) fn new(value: JsValue) -> Self {
        Self(value)
    }
    pub(super) fn below(&self, area: f64) -> Result<bool, JsValue> {
        below(area, 1.0, &self.0)
    }
    pub(super) fn check(&self, width: f64, height: f64) -> Result<(), JsValue> {
        if above(width, height, &self.0)? {
            Err(js_sys::Error::new("image exceeds pixel limit").into())
        } else {
            Ok(())
        }
    }
}
