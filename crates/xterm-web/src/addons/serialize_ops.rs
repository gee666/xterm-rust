//! JS operators at the addon ABI. These contain no terminal or serializer logic.
use js_sys::JsString;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function serialOptional(value, key) { return value?.[key]; }
export function serialProperty(value, key) { return value[key]; }
export function serialTruthy(value) { return !!value; }
export function serialNumber(value) { return +value; }
export function serialStart(range) { return typeof range.start === 'number' ? range.start : range.start.line; }
export function serialEnd(range) { return typeof range.end === 'number' ? range.end : range.end.line; }
export function serialHtmlStart(range) { return typeof range.startLine === 'number' ? range.startLine : range.startLine; }
export function serialHtmlEnd(range) { return typeof range.endLine === 'number' ? range.endLine : range.endLine; }
export function serialRetained(value, rows, length) { return value === undefined ? length : Math.max(0, Math.min(value + rows, length)); }
export function serialSpan(start, end) {
  const span = end - start;
  if (typeof span === 'number' && (!Number.isInteger(span) || span < 0 || span > 0xffffffff)) throw new RangeError('Invalid array length');
  if (typeof start === 'bigint') return 0 + start;
  return span;
}
")]
extern "C" {
    #[wasm_bindgen(catch)]
    pub(super) fn serialOptional(value: &JsValue, key: &JsString) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    pub(super) fn serialProperty(value: &JsValue, key: &JsString) -> Result<JsValue, JsValue>;
    pub(super) fn serialTruthy(value: &JsValue) -> bool;
    #[wasm_bindgen(catch)]
    pub(super) fn serialNumber(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch)]
    pub(super) fn serialStart(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    pub(super) fn serialEnd(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    pub(super) fn serialHtmlStart(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    pub(super) fn serialHtmlEnd(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    pub(super) fn serialRetained(value: &JsValue, rows: f64, length: f64) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch)]
    pub(super) fn serialSpan(start: &JsValue, end: &JsValue) -> Result<f64, JsValue>;
}

pub(super) fn optional(value: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    serialOptional(value, &key.into())
}
pub(super) fn property(value: &JsValue, key: &str) -> Result<JsValue, JsValue> {
    serialProperty(value, &key.into())
}
