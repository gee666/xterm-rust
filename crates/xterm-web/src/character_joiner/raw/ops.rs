//! JavaScript operators only. Joining policy stays in Rust; accesses intentionally
//! repeat where upstream repeats them, since getters can mutate or throw.
//! Math lookup and its operand reads share one call because an operand getter
//! can replace Math.min/Math.max after the source has resolved the function.
use wasm_bindgen::prelude::*;
#[wasm_bindgen(inline_js = r#"
export function joinerGet(v,k) { return v[k]; }
export function joinerSet(v,k,x) { 'use strict'; v[k]=x; }
export function joinerLess(a,b) { return a<b; }
export function joinerLessEqual(a,b) { return a<=b; }
export function joinerMinStart(a,b) { return Math.min(a[0],b[0]); }
export function joinerMaxEnd(a,b) { return Math.max(a[1],b[1]); }
export function joinerSubtractOne(a) { return a-1; }
export function joinerInsert(v,i,x) { v.splice(i,0,x); }
export function joinerRemove(v,i) { v.splice(i,1); }
export function joinerPush(v,x) { v.push(x); }
export function joinerPushFrom(target,source,index) { target.push(source[index]); }
export function joinerReport(e) { console.error(e); }
export function joinerCall(record,text) { return record.handler(text); }
export function joinerRecord(id,handler) { return {id,handler}; }
"#)]
extern "C" {
    #[wasm_bindgen(js_name = joinerRecord)]
    pub fn record(id: f64, handler: &js_sys::Function) -> JsValue;
    #[wasm_bindgen(catch, js_name = joinerCall)]
    pub fn call(record: &JsValue, text: &js_sys::JsString) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = joinerReport)]
    pub fn report(e: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = joinerGet)]
    pub fn get(v: &JsValue, k: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = joinerSet)]
    pub fn set(v: &JsValue, k: &JsValue, x: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = joinerLess)]
    pub fn less(a: &JsValue, b: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = joinerLessEqual)]
    pub fn le(a: &JsValue, b: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = joinerMinStart)]
    pub fn min_start(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = joinerMaxEnd)]
    pub fn max_end(a: &JsValue, b: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = joinerSubtractOne)]
    pub fn subtract_one(a: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = joinerInsert)]
    pub fn insert(v: &JsValue, i: f64, x: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = joinerRemove)]
    pub fn remove(v: &JsValue, i: f64) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = joinerPushFrom)]
    pub fn push_from(target: &JsValue, source: &JsValue, index: f64) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = joinerPush)]
    pub fn push(v: &JsValue, x: &JsValue) -> Result<(), JsValue>;
}
pub fn at(v: &JsValue, index: f64) -> Result<JsValue, JsValue> {
    get(v, &index.into())
}
pub fn length(v: &JsValue) -> Result<JsValue, JsValue> {
    get(v, &"length".into())
}
pub fn put(v: &JsValue, index: f64, x: &JsValue) -> Result<(), JsValue> {
    set(v, &index.into(), x)
}
