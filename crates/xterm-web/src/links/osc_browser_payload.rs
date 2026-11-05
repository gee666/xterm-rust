// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
//! Host objects for Rust-discovered OSC links. JS owns the range and callback
//! edges, so cached payloads and callback arguments share identity without a
//! pointer-key cache or a Rust closure cycle back to the link/controller.
use super::{browser::default_activate, browser_linkifier::BrowserLink, Link};
use crate::utf16::{from_js_string, to_js_string};
use js_sys::{Array, JsString};
use wasm_bindgen::prelude::*;
use web_sys::Window;

#[wasm_bindgen(module = "/src/links/raw_osc_bridge.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = oscBrowserLink)]
    fn browser_link(
        text: &JsString,
        coordinates: &Array,
        handler: &JsValue,
        fallback: &JsValue,
    ) -> Result<JsValue, JsValue>;
}

pub(super) fn link(data: Link, handler: &JsValue, window: &Window) -> Result<BrowserLink, JsValue> {
    let coordinates = Array::of4(
        &JsValue::from_f64(data.range.start.x as f64),
        &JsValue::from_f64(data.range.start.y as f64),
        &JsValue::from_f64(data.range.end.x as f64),
        &JsValue::from_f64(data.range.end.y as f64),
    );
    let fallback = if handler.is_truthy() {
        JsValue::UNDEFINED
    } else {
        let window = window.clone();
        // Only the browser window is retained. JS GC owns this callback, which
        // never holds a terminal, provider, link or controller alive.
        Closure::<dyn Fn(JsValue, JsString) -> Result<(), JsValue>>::new(move |_, text| {
            default_activate(&window, &from_js_string(&text))
        })
        .into_js_value()
    };
    browser_link(&to_js_string(&data.text), &coordinates, handler, &fallback)
        .map(BrowserLink::JavaScript)
}
