// Copyright (c) 2023 The xterm.js authors. All rights reserved.
// Licensed under MIT. BrowserClipboardProvider adapter.
use crate::utf16::{from_js_string, to_js_string};
use js_sys::JsString;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::Clipboard;
use xterm_addons::clipboard::{ClipboardError, ClipboardProvider, Completion};

pub struct BrowserClipboardProvider {
    clipboard: Clipboard,
}
impl BrowserClipboardProvider {
    pub fn new(clipboard: Clipboard) -> Self {
        Self { clipboard }
    }
}
fn error(value: JsValue) -> ClipboardError {
    ClipboardError(format!("{value:?}"))
}
impl ClipboardProvider for BrowserClipboardProvider {
    fn read_text(&self, _selection: &[u16]) -> Completion<Vec<u16>> {
        let promise = self.clipboard.read_text();
        Completion::Pending(Box::pin(async move {
            let value = JsFuture::from(promise).await.map_err(error)?;
            let text = value.dyn_into::<JsString>().map_err(error)?;
            Ok(from_js_string(&text))
        }))
    }
    fn write_text(&self, _selection: &[u16], text: &[u16]) -> Completion<()> {
        // web-sys write_text takes &str, which cannot represent lone surrogates.
        // Call the same built-in with a raw JS string instead.
        let result = js_sys::Reflect::get(self.clipboard.as_ref(), &"writeText".into())
            .and_then(|method| method.dyn_into::<js_sys::Function>())
            .and_then(|method| method.call1(self.clipboard.as_ref(), &to_js_string(text)))
            .and_then(|value| value.dyn_into::<js_sys::Promise>());
        match result {
            Err(value) => Completion::Ready(Err(error(value))),
            Ok(promise) => Completion::Pending(Box::pin(async move {
                JsFuture::from(promise).await.map_err(error)?;
                Ok(())
            })),
        }
    }
}
