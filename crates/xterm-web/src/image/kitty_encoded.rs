// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Blob capture and reads start before the native controller releases BASE64.
use super::js;
use std::{any::Any, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use xterm_addons::image::{
    encoded::{ByteRead, EncodedBytes, EncodedSource, EncodedSourceFactory},
    storage::StorageError,
};

pub(super) struct BlobFactory;
pub(super) struct BlobSource {
    blob: JsValue,
    length: usize,
    mime: String,
}
impl BlobSource {
    pub(super) fn value(&self) -> JsValue {
        self.blob.clone()
    }
}
impl EncodedSourceFactory for BlobFactory {
    fn capture_owned(&self, data: Vec<u8>, mime: &str) -> Result<EncodedBytes, StorageError> {
        let parts = js_sys::Array::new();
        parts.push(&js_sys::Uint8Array::from(data.as_slice()));
        let options = js_sys::Object::new();
        js::set(&options, "type", &mime.into()).map_err(js::error)?;
        let blob = js::construct("Blob", &[parts.into(), options.into()]).map_err(js::error)?;
        Ok(EncodedBytes::from_source(Rc::new(BlobSource {
            blob,
            length: data.len(),
            mime: mime.to_owned(),
        })))
    }
}
impl EncodedSource for BlobSource {
    fn size(&self) -> usize {
        self.length
    }
    fn mime(&self) -> &str {
        &self.mime
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn read(self: Rc<Self>) -> ByteRead {
        // Calling arrayBuffer is source-eager even though its result is awaited.
        let pending = js::call(&self.blob, "arrayBuffer", &[]);
        Box::pin(async move {
            let result = await_value(pending.map_err(js::error)?)
                .await
                .map_err(js::error)?;
            Ok(js_sys::Uint8Array::new(&result).to_vec())
        })
    }
}
pub(super) async fn await_value(value: JsValue) -> Result<JsValue, JsValue> {
    // Await accepts thenables and plain values, not just native Promises.
    JsFuture::from(js_sys::Promise::resolve(&value)).await
}
pub(super) fn bitmap_call(args: &[JsValue]) -> Result<JsValue, JsValue> {
    let method = js::function(
        js::get(&js_sys::global(), "createImageBitmap")?,
        "createImageBitmap",
    )?;
    method.apply(
        &JsValue::UNDEFINED,
        &args.iter().cloned().collect::<js_sys::Array>(),
    )
}
pub(super) fn byte_view(value: &JsValue) -> Result<Vec<u8>, JsValue> {
    value
        .dyn_ref::<js_sys::Uint8Array>()
        .map(js_sys::Uint8Array::to_vec)
        .ok_or_else(|| js_sys::TypeError::new("DecompressionStream returned non-byte data").into())
}
