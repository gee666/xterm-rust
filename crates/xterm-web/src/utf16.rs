//! Lossless JavaScript strings at the WASM boundary.
//!
//! wasm-bindgen's String conversion replaces lone surrogates. Terminal payloads
//! must instead use JsString and raw UTF-16 code units.

use js_sys::{Array, JsString};

/// Keep variadic String.fromCharCode calls below browser argument-count limits.
/// This is an implementation chunk size, not a terminal payload limit.
const CHAR_CODE_CHUNK: usize = 4096;

pub fn from_js_string(value: &JsString) -> Vec<u16> {
    value.iter().collect()
}

pub fn to_js_string(value: &[u16]) -> JsString {
    let chunks = Array::new();
    for chunk in value.chunks(CHAR_CODE_CHUNK) {
        chunks.push(&JsString::from_char_code(chunk));
    }
    chunks.join("")
}
