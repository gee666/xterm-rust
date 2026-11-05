// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! JavaScript callbacks share their link's live range with the Rust controller.
//! The typed provider adapter remains separate for native Rust callback users.
use super::provider::BrowserWebLinkTerminal;
use crate::{
    links::browser_linkifier::{BrowserLink, BrowserLinkProvider, LinkReply},
    utf16::{from_js_string, to_js_string},
};
use js_sys::{Array, JsString};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::MouseEvent;
use xterm_addons::{
    web_link_provider::{Provider, WebLinkTerminal},
    web_links::{LinkBuffer, LinkError},
};
use xterm_core::lifecycle::DisposableHandle;

#[wasm_bindgen(module = "/src/utility_addons/raw_link_bridge.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = webBrowserLink)]
    fn browser_link(
        text: &JsString,
        coordinates: &Array,
        handler: &JsValue,
        options: &JsValue,
    ) -> Result<JsValue, JsValue>;
}

pub(super) struct JavaScriptWebLinkTerminal {
    terminal: BrowserWebLinkTerminal,
    handler: JsValue,
    options: JsValue,
}
pub(super) fn handler_or_default(handler: JsValue) -> JsValue {
    if handler.is_undefined() {
        // GC owns this callback. It retains no terminal, provider or link.
        Closure::<dyn Fn(JsValue, JsString) -> Result<(), JsValue>>::new(move |_, text| {
            super::open_link::open_link(&from_js_string(&text))
                .map_err(|error| crate::js_error::to_js_error(&error))
        })
        .into_js_value()
    } else {
        handler
    }
}
impl JavaScriptWebLinkTerminal {
    pub fn new(terminal: BrowserWebLinkTerminal, handler: JsValue, options: JsValue) -> Self {
        Self {
            terminal,
            handler,
            options,
        }
    }
}
struct JavaScriptProvider {
    provider: Provider<MouseEvent>,
    handler: JsValue,
    options: JsValue,
}
impl BrowserLinkProvider for JavaScriptProvider {
    fn provide_links(&self, y: usize, callback: LinkReply) -> Result<(), JsValue> {
        let links = (self.provider)(y).map_err(|error| crate::js_error::to_js_error(&error))?;
        let mut result = Vec::with_capacity(links.len());
        for link in links {
            let coordinates = Array::of4(
                &JsValue::from_f64(link.range.start.x as f64),
                &JsValue::from_f64(link.range.start.y as f64),
                &JsValue::from_f64(link.range.end.x as f64),
                &JsValue::from_f64(link.range.end.y as f64),
            );
            result.push(BrowserLink::JavaScript(browser_link(
                &to_js_string(&link.text),
                &coordinates,
                &self.handler,
                &self.options,
            )?));
        }
        callback(Some(result))
    }
}
impl WebLinkTerminal<MouseEvent> for JavaScriptWebLinkTerminal {
    fn buffer(&self) -> Result<Box<dyn LinkBuffer>, LinkError> {
        self.terminal.buffer()
    }
    fn register_link_provider(
        &self,
        provider: Provider<MouseEvent>,
    ) -> Result<DisposableHandle, LinkError> {
        Ok(Rc::new(self.terminal.providers.register(Rc::new(
            JavaScriptProvider {
                provider,
                handler: self.handler.clone(),
                options: self.options.clone(),
            },
        ))))
    }
    fn open_link(&self, event: &MouseEvent, uri: &[u16]) -> Result<(), LinkError> {
        self.terminal.open_link(event, uri)
    }
}
