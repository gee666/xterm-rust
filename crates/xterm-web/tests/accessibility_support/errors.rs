// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use wasm_bindgen::prelude::*;
#[wasm_bindgen(
    inline_js = "export function accessibilitySelectionError(doc){let seen;const capture=e=>{seen=e.error;e.preventDefault();e.stopImmediatePropagation();};const targets=[...new Set([window,doc.defaultView].filter(Boolean))];for(const target of targets)target.addEventListener('error',capture,true);try{doc.dispatchEvent(new (doc.defaultView??window).Event('selectionchange'));}finally{for(const target of targets)target.removeEventListener('error',capture,true);}return seen;}"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = accessibilitySelectionError)]
    pub fn selection_error(document: &web_sys::Document) -> Result<JsValue, JsValue>;
}
