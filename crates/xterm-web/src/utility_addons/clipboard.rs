//! Typed Rust clipboard-provider contract over the same exported browser host.
pub use super::clipboard_bindings::BrowserClipboardProvider;
use crate::utf16::{from_js_string, to_js_string};
use js_sys::{JsString, Promise};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use xterm_addons::clipboard::{ClipboardError, ClipboardProvider, Completion};
fn error(value: JsValue) -> ClipboardError {
    ClipboardError::host(crate::js_error::JsCallbackError(value))
}
impl ClipboardProvider for BrowserClipboardProvider {
    fn read_text(&self, selection: &[u16]) -> Completion<Vec<u16>> {
        match self.read_text_js(to_js_string(selection).into()) {
            Err(value) => Completion::Ready(Err(error(value))),
            Ok(value) => Completion::Pending(Box::pin(async move {
                let value = JsFuture::from(Promise::resolve(&value))
                    .await
                    .map_err(error)?;
                let text = value.dyn_into::<JsString>().map_err(error)?;
                Ok(from_js_string(&text))
            })),
        }
    }
    fn write_text(&self, selection: &[u16], text: &[u16]) -> Completion<()> {
        match self.write_text_js(to_js_string(selection).into(), to_js_string(text).into()) {
            Err(value) => Completion::Ready(Err(error(value))),
            Ok(value) => Completion::Pending(Box::pin(async move {
                JsFuture::from(Promise::resolve(&value))
                    .await
                    .map_err(error)?;
                Ok(())
            })),
        }
    }
}
