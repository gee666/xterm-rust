//! Lossless JavaScript strings at the WASM boundary.
//!
//! wasm-bindgen's String conversion replaces lone surrogates. Terminal payloads
//! must instead use JsString and raw UTF-16 code units.

use js_sys::JsString;
pub use xterm_wasm_support::utf16::{from_js_string, to_js_string};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(
    inline_js = "export function xtermSetTextContent(node, text) { 'use strict'; node.textContent = text; }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = xtermSetTextContent)]
    fn assign_text_content(node: &web_sys::Node, text: &JsString) -> Result<(), JsValue>;
}

/// Preserve strict assignment failures and raw UTF-16 without a Rust String.
/// Release caller-owned RefCell guards before entry, since host setters may reenter.
pub fn set_text_content(node: &web_sys::Node, text: JsString) -> Result<(), JsValue> {
    assign_text_content(node, &text)
}

