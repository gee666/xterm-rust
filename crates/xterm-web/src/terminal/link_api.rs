//! Public provider registration shares the terminal's existing priority registry.
use super::Terminal;
use crate::links::{
    browser_linkifier::{BrowserLink, BrowserLinkProvider, LazyLinkReply, LinkReply},
    browser_payload::{call, map, property},
};
use js_sys::Array;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};

#[wasm_bindgen(module = "/src/links/raw_link_bridge.js")]
extern "C" {
    #[wasm_bindgen(js_name = linkProviderToken)]
    fn provider_token(provider: &JsValue, callback: &JsValue) -> JsValue;
}

struct JavaScriptProvider(JsValue);
impl BrowserLinkProvider for JavaScriptProvider {
    fn provide_links(&self, y: usize, callback: LinkReply) -> Result<(), JsValue> {
        self.provide_links_lazy(y, Box::new(move |resolve| callback(resolve()?)))
    }
    fn provide_links_lazy(&self, y: usize, callback: LazyLinkReply) -> Result<(), JsValue> {
        let callback =
            Closure::<dyn Fn(JsValue) -> Result<(), JsValue>>::new(move |links: JsValue| {
                callback(Box::new(move || {
                    let links = map(&links)?;
                    if links.is_undefined() || links.is_null() {
                        return Ok(None);
                    }
                    let links: Array = links.unchecked_into();
                    Ok(Some(links.iter().map(BrowserLink::JavaScript).collect()))
                }))
            })
            .into_js_value();
        call(
            &property(&self.0, "provideLinks")?,
            &self.0,
            &Array::of2(&JsValue::from_f64(y as f64), &callback),
        )?;
        Ok(())
    }
    fn javascript_identity(&self) -> Option<&JsValue> {
        Some(&self.0)
    }
}

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(js_name = registerLinkProvider)]
    pub fn register_link_provider(&self, provider: JsValue) -> JsValue {
        self.shared
            .link_providers
            .register(Rc::new(JavaScriptProvider(provider.clone())));
        let weak = Rc::downgrade(&self.shared.link_providers);
        let dispose = Closure::<dyn Fn(JsValue)>::new(move |provider| {
            let Some(providers) = weak.upgrade() else {
                return;
            };
            // Resolve identity at disposal, including registrations made after
            // this token removed every previous registration of the JS object.
            let first = providers
                .snapshot()
                .into_iter()
                .find(|entry| entry.javascript_identity() == Some(&provider));
            if let Some(first) = first {
                providers.unregister(&first);
            }
        })
        .into_js_value();
        // Keep the provider edge on the JS side. provider -> token cycles must
        // not retain a Rust closure that in turn strongly owns the JS provider.
        provider_token(&provider, &dispose)
    }
}
