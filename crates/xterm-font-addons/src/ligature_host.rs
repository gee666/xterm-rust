//! Minimal DOM/JavaScript coercion boundaries for the Rust ligature addon.
use js_sys::{Array, JsString};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(module = "/src/ligature_host.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = sortedFallbacks)]
    pub fn sorted_fallbacks(options: &JsValue, defaults: &Array) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = fontFeatures)]
    pub fn features(options: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = terminalElement)]
    pub fn element(terminal: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = registerJoiner)]
    pub fn register(terminal: &JsValue, callback: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = deregisterJoiner)]
    pub fn deregister(terminal: &JsValue, id: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = setFontFeatures)]
    pub fn set_features(terminal: &JsValue, value: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = clearFontFeatures)]
    pub fn clear_features(terminal: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = fontFamily)]
    pub fn family(terminal: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = debugEnabled)]
    pub fn debug(terminal: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = refreshFont)]
    pub fn refresh(terminal: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = fontFailure)]
    pub fn failure(previous: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = finishFontLoad)]
    pub fn finish_load(
        promise: &js_sys::Promise,
        loaded: &JsValue,
        failed: &JsValue,
    ) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = fallbackInBounds)]
    pub fn in_bounds(patterns: &JsValue, index: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = fallbackPattern)]
    pub fn pattern(patterns: &JsValue, index: f64) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = startsWith)]
    pub fn starts_with(text: &JsString, pattern: &JsValue, index: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = fallbackEnd)]
    pub fn end(patterns: &JsValue, pattern_index: f64, text_index: f64)
        -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = fallbackNext)]
    pub fn next(patterns: &JsValue, pattern_index: f64, text_index: f64) -> Result<f64, JsValue>;
}
