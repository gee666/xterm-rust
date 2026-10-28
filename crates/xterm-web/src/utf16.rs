//! Lossless JavaScript strings at the WASM boundary.
//!
//! wasm-bindgen's String conversion replaces lone surrogates. Terminal payloads
//! must instead use JsString and raw UTF-16 code units.

use js_sys::JsString;
use wasm_bindgen::prelude::*;
pub use xterm_wasm_support::utf16::{from_js_string, to_js_string};

#[wasm_bindgen(
    inline_js = "export function xtermAssignProperty(target, key, value) { 'use strict'; target[key] = value; }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = xtermAssignProperty)]
    fn assign_property(target: &JsValue, key: &str, value: &JsValue) -> Result<(), JsValue>;
}

/// Preserve strict assignment failures and raw UTF-16 without a Rust String.
/// Release caller-owned RefCell guards before entry, since host setters may reenter.
pub fn set_text_content(node: &web_sys::Node, text: JsString) -> Result<(), JsValue> {
    set_property(node.as_ref(), "textContent", text.as_ref())
}

/// A strict platform property assignment retaining its original exception.
/// This accepts raw host objects as well as strings, so WebIDL conversion runs
/// inside the selected setter rather than prematurely at a Rust boundary.
pub fn set_property(target: &JsValue, key: &str, value: &JsValue) -> Result<(), JsValue> {
    assign_property(target, key, value)
}
