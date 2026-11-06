//! JavaScript ABI operations not expressible by Rust's numeric types. These
//! helpers preserve property access, ECMAScript coercion and host calls. The addon
//! state machine, validation decisions and events remain in Rust. The public
//! Progress setter must call the current host Math methods, including replaced
//! getters, receivers and arbitrary return values, rather than emulate a clamp.
//! Static wasm-bindgen snippets avoid requiring eval or unsafe-eval CSP rules.
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function statusProperty(value, key) { return value[key]; }
export function statusLess(left, right) { return left < right; }
export function statusProgressValue(progress) { return Math.min(Math.max(progress.value, 0), 100); }
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = statusProperty)]
    pub fn property(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = statusLess)]
    pub fn less(left: &JsValue, right: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = statusProgressValue)]
    pub fn progress_value(progress: &JsValue) -> Result<JsValue, JsValue>;
}
