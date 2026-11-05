//! Host operators only. Row traversal and joining policy remain in Rust.
use wasm_bindgen::prelude::*;
#[wasm_bindgen(inline_js = r#"
export function rowGet(v,k) { return v[k]; }
export function rowShift(v) { return v.shift(); }
export function rowLess(a,b) { return a<b; }
export function rowLessEqual(a,b) { return a<=b; }
export function rowAdd(a,b) { return a+b; }
export function rowIncrement(a) { return ++a; }
export function rowSubtract(a,b) { return a-b; }
export function rowSubtractOne(a) { return a-1; }
export function rowMultiply(a,b) { return a*b; }
export function rowAddress(a) { return a*3; }
export function rowMin(a,b) { return Math.min(a,b); }
"#)]
extern "C" {
    #[wasm_bindgen(catch, js_name = rowGet)]
    pub fn get(value: &JsValue, key: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = rowShift)]
    pub fn shift(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = rowLess)]
    pub fn less(a: &JsValue, b: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = rowLessEqual)]
    pub fn le(a: &JsValue, b: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = rowAdd)]
    pub fn add(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = rowIncrement)]
    pub fn increment(a: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = rowSubtractOne)]
    pub fn subtract_one(a: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = rowSubtract)]
    pub fn difference(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = rowMultiply)]
    pub fn multiply(a: &JsValue, b: f64) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = rowAddress)]
    pub fn address(a: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = rowMin)]
    pub fn min(a: &JsValue, b: f64) -> Result<JsValue, JsValue>;
}
pub fn at(value: &JsValue, index: f64) -> Result<JsValue, JsValue> {
    get(value, &index.into())
}
