//! JavaScript ABI operations not expressible by Rust's numeric types. These
//! helpers only access properties and perform ECMAScript coercion. The addon
//! state machine, validation decisions, clamping and events remain in Rust.
//! Static wasm-bindgen snippets avoid requiring eval or unsafe-eval CSP rules.
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function statusProperty(value, key) { return value[key]; }
export function statusLess(left, right) { return left < right; }
export function statusNumber(value) { return +value; }
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = statusProperty)]
    pub fn property(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = statusLess)]
    pub fn less(left: &JsValue, right: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = statusNumber)]
    pub fn number(value: &JsValue) -> Result<f64, JsValue>;
}
