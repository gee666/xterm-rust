//! A live JavaScript byte view shared by WriteBuffer and the bounded parser.
//! No full-input copy occurs at enqueue or before a paused parser resumes.

use crate::js_error::JsCallbackError;
use js_sys::{JsString, Uint8Array};
use std::{borrow::Cow, ops::Range, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::{
    input::{parse_source::{ParseChunk, ParseInputSource, ParseSourceError}, write_buffer::{WriteError, WritePayload}},
    log_service::{LogValue},
};

#[wasm_bindgen(inline_js = "
export function byteLength(value) { return value.length; }
function copyBytes(value) {
  const length = value.length;
  const result = new Uint8Array(length);
  for (let i = 0; i < length; ++i) result[i] = value[i];
  return result;
}
export function allBytes(value) { return copyBytes(value); }
export function rangeBytes(value, start, end) { return copyBytes(value.subarray(start, end)); }
export function debugBytes(value) {
  return Array.prototype.map.call(value, e => String.fromCharCode(e)).join('');
}
export function completeWrite(callback) { callback(); }
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = byteLength)]
    fn byte_length(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch, js_name = allBytes)]
    fn all_bytes(value: &JsValue) -> Result<Vec<u8>, JsValue>;
    #[wasm_bindgen(catch, js_name = rangeBytes)]
    fn range_bytes(value: &JsValue, start: usize, end: usize) -> Result<Vec<u8>, JsValue>;
    #[wasm_bindgen(catch, js_name = debugBytes)]
    fn debug_bytes(value: &JsValue) -> Result<JsString, JsValue>;
    #[wasm_bindgen(catch, js_name = completeWrite)]
    pub(crate) fn complete_write(callback: &JsValue) -> Result<(), JsValue>;
}

#[derive(Debug)]
pub struct ByteView(pub JsValue);
impl ByteView {
    pub fn from_view(view: Uint8Array) -> Rc<Self> { Rc::new(Self(view.into())) }
    fn length(&self) -> Result<usize, JsValue> {
        let length = byte_length(&self.0)?;
        // A real Uint8Array has an unsigned integral length. Check before Rust's
        // conversion rather than silently saturating a malformed host result.
        if !length.is_finite() || length < 0.0 || length.fract() != 0.0 || length > usize::MAX as f64 {
            return Err(js_sys::RangeError::new("Invalid byte view length").into());
        }
        Ok(length as usize)
    }
}
impl WritePayload for ByteView {
    fn len(&self) -> Result<usize, WriteError> { self.length().map_err(|error| WriteError::external(JsCallbackError(error))) }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn into_parse_source(self: Rc<Self>) -> Option<Rc<dyn ParseInputSource>> { Some(self) }
}
impl ParseInputSource for ByteView {
    fn len(&self) -> Result<usize, ParseSourceError> { self.length().map_err(|error| ParseSourceError::new(JsCallbackError(error))) }
    fn read_range(&self, range: Range<usize>) -> Result<ParseChunk<'_>, ParseSourceError> {
        range_bytes(&self.0, range.start, range.end).map(|bytes| ParseChunk::Bytes(Cow::Owned(bytes)))
            .map_err(|error| ParseSourceError::new(JsCallbackError(error)))
    }
    fn read_all(&self) -> Result<ParseChunk<'_>, ParseSourceError> {
        all_bytes(&self.0).map(|bytes| ParseChunk::Bytes(Cow::Owned(bytes)))
            .map_err(|error| ParseSourceError::new(JsCallbackError(error)))
    }
    fn debug_units(&self) -> Result<Vec<u16>, ParseSourceError> {
        debug_bytes(&self.0).map(|text| xterm_wasm_support::utf16::from_js_string(&text))
            .map_err(|error| ParseSourceError::new(JsCallbackError(error)))
    }
    fn trace_value(&self) -> Result<LogValue, ParseSourceError> {
        xterm_wasm_support::options::from_js(&self.0).map(LogValue::Value)
            .map_err(|error| ParseSourceError::new(JsCallbackError(error)))
    }
}
