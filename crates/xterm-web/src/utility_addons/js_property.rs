//! ECMAScript property access boxes primitives, unlike Reflect.get. Keep that
//! language operation at the binding boundary and preserve thrown JS values.
use js_sys::{Array, Uint8Array};
use wasm_bindgen::prelude::*;
#[wasm_bindgen(
    inline_js = "export function addonProperty(value, key) { return value[key]; } export function addonSet(value,key,next) { value[key]=next; } export function addonCall(value,key,args) { return value[key](...args); } export function addonBytes(value) { return new Uint8Array(value); }"
)]
extern "C" {
    #[wasm_bindgen(catch,js_name=addonProperty)]
    pub fn get(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=addonSet)]
    pub fn set(value: &JsValue, key: &str, next: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch,js_name=addonCall)]
    pub fn call(value: &JsValue, key: &str, args: &Array) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=addonBytes)]
    pub fn bytes(value: &JsValue) -> Result<Uint8Array, JsValue>;
}
