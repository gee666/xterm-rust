//! Value serialization of real line state, excluding opaque WASM handles.
use super::WebBufferLine;
use crate::{js_error::to_js_error, utf16::to_js_string};
use js_sys::{Array, Uint32Array};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
export function lineJSON(data, combined, extended, length, wrapped, valid, cache, trimmed) {
  return {_combined:Object.fromEntries(combined),_extendedAttrs:Object.fromEntries(extended),
    _cacheValid:valid,_cache:cache,_cacheTrimmed:trimmed,_data:data,length,_isWrapped:wrapped};
}
export function extendedJSON(raw, url, payload) {return {_ext:raw,payload,_urlId:url};}
")]
extern "C" {
    #[wasm_bindgen(catch,js_name=lineJSON)]
    fn line_json(
        data: &Uint32Array,
        combined: &Array,
        extended: &Array,
        length: usize,
        wrapped: bool,
        valid: bool,
        cache: &js_sys::JsString,
        trimmed: bool,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=extendedJSON)]
    fn extended_json(raw: u32, url: f64, payload: &JsValue) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen(js_class=BufferLine)]
impl WebBufferLine {
    #[wasm_bindgen(js_name=toJSON)]
    pub fn to_json(&self) -> Result<JsValue, JsValue> {
        let snapshot = self
            .native
            .try_borrow()
            .map_err(|error| to_js_error(&error))?
            .snapshot();
        let words: Vec<u32> = snapshot
            .words
            .iter()
            .flat_map(|cell| cell.iter().copied())
            .collect();
        let data = Uint32Array::from(words.as_slice());
        let combined = Array::new();
        for (index, text) in &snapshot.combined {
            combined.push(&Array::of2(&index.to_string().into(), &to_js_string(text)));
        }
        let extended = Array::new();
        for (index, attrs) in &snapshot.extended {
            let payload = match attrs.payload() {
                None => JsValue::UNDEFINED,
                Some(value) => value.downcast_ref::<JsValue>().cloned().ok_or_else(|| {
                    js_sys::TypeError::new(
                        "Native attribute payload has no JavaScript representation",
                    )
                })?,
            };
            let value = extended_json(attrs.raw_ext(), attrs.url_id() as f64, &payload)?;
            extended.push(&Array::of2(&index.to_string().into(), &value));
        }
        let (valid, text, trimmed) = match snapshot.cache {
            Some((text, trimmed)) => (true, text, trimmed),
            None => (false, Vec::new(), false),
        };
        line_json(
            &data,
            &combined,
            &extended,
            snapshot.length,
            snapshot.is_wrapped,
            valid,
            &to_js_string(&text),
            trimmed,
        )
    }
}
