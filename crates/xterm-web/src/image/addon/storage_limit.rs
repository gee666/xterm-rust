// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Keep JavaScript relational/division coercion at the binding boundary. Each
//! operation may call user code; a single eager Number conversion is not equal.
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
const FALLBACK_PIXEL_LIMIT = 2500000;
const PIXELS_PER_MEGABYTE = 250000;
export function imageStoragePixels(value) {
  if (value < 0.5 || value > 1000) {
    throw RangeError('invalid storageLimit, should be at least 0.5 MB and not exceed 1G');
  }
  return (value / 4 * 1000000) >>> 0;
}
export function imageStorageInitialPixels(value) {
  try { return imageStoragePixels(value); }
  catch (error) {
    if (error instanceof Error) console.error(error.message);
    console.warn(`storageLimit is set to ${FALLBACK_PIXEL_LIMIT / PIXELS_PER_MEGABYTE} MB`);
    return FALLBACK_PIXEL_LIMIT;
  }
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageStoragePixels)]
    pub(super) fn pixels(value: &JsValue) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = imageStorageInitialPixels)]
    pub(super) fn initial_pixels(value: &JsValue) -> Result<u32, JsValue>;
}
