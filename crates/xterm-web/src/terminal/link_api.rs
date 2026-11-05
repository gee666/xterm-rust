//! Public provider registration shares the terminal's existing priority registry.
use super::Terminal;
use crate::links::{
    browser_linkifier::{BrowserLink, BrowserLinkProvider, LazyLinkReply, LinkReply},
    browser_payload::{call, map, property},
};
use js_sys::Array;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, prelude::*};
use xterm_core::lifecycle::Disposable;

struct JavaScriptProvider(JsValue);
impl BrowserLinkProvider for JavaScriptProvider {
    fn provide_links(&self, y: usize, callback: LinkReply) -> Result<(), JsValue> {
        self.provide_links_lazy(y, Box::new(move |resolve| callback(resolve()?)))
    }
    fn provide_links_lazy(&self, y: usize, callback: LazyLinkReply) -> Result<(), JsValue> {
        let callback = Closure::<dyn Fn(JsValue) -> Result<(), JsValue>>::new(move |links: JsValue| {
            callback(Box::new(move || {
                let links = map(&links)?;
                if links.is_undefined() || links.is_null() { return Ok(None); }
                let links = Array::from(&links);
                Ok(Some(links.iter().map(BrowserLink::JavaScript).collect()))
            }))
        }).into_js_value();
        call(&property(&self.0, "provideLinks")?, &self.0, &Array::of2(&JsValue::from_f64(y as f64), &callback))?;
        Ok(())
    }
    fn javascript_identity(&self) -> Option<&JsValue> { Some(&self.0) }
}

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(js_name = registerLinkProvider)]
    pub fn register_link_provider(&self, provider: JsValue) -> JsValue {
        // Source indexOf removes the first registration of the same JS object,
        // including repeated disposal of a token for duplicate registrations.
        let existing = self.shared.link_providers.snapshot().into_iter().find(|entry| entry.javascript_identity() == Some(&provider));
        let provider = existing.unwrap_or_else(|| Rc::new(JavaScriptProvider(provider)));
        let registration = self.shared.link_providers.register(provider);
        xterm_wasm_support::events::from_callback(move || {
            registration.dispose();
            Ok(JsValue::UNDEFINED)
        })
    }
}
