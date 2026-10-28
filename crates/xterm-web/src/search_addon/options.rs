// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use js_sys::JsString;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use xterm_addons::search::{
    SearchAddonError, SearchColor, SearchDecorationOptionsHandle, SearchDecorationSource,
    SearchHighlightLimit, SearchOptions, SearchOptionsSource,
};

// These access/coercion operations need JS boxing, getter exceptions, BigInt and
// Symbol semantics. All matching, traversal, result state and decoration layout
// remain Rust. Static module imports also work under a no-eval CSP.
#[wasm_bindgen(
    inline_js = "export function searchAddonGet(v,k){return v?.[k];}\nexport function searchAddonLimitReached(v,n){return n>=v;}\nexport function searchAddonLimitEnd(v,n){let x=+v;if(Number.isNaN(x))x=0;x=Math.trunc(x);return x<0?Math.max(n+x,0):Math.min(x,n);}"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = searchAddonGet)]
    pub fn property(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = searchAddonLimitReached)]
    fn reached(value: &JsValue, count: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = searchAddonLimitEnd)]
    fn end(value: &JsValue, length: f64) -> Result<f64, JsValue>;
}
fn host_error(value: JsValue) -> SearchAddonError {
    SearchAddonError::Host(Rc::new(crate::js_error::JsCallbackError(value)))
}
pub struct JsLimit(pub JsValue);
impl SearchHighlightLimit for JsLimit {
    fn reached(&self, count: usize) -> Result<bool, SearchAddonError> {
        reached(&self.0, count as f64).map_err(host_error)
    }
    fn end_index(&self, length: usize) -> Result<usize, SearchAddonError> {
        end(&self.0, length as f64)
            .map(|end| end as usize)
            .map_err(host_error)
    }
}
struct JsDecorations(JsValue);
impl SearchDecorationSource for JsDecorations {
    fn color(&self, kind: SearchColor) -> Result<Option<String>, xterm_core::event::EmitError> {
        let key = match kind {
            SearchColor::MatchBackground => "matchBackground",
            SearchColor::MatchBorder => "matchBorder",
            SearchColor::MatchOverview => "matchOverviewRuler",
            SearchColor::ActiveBackground => "activeMatchBackground",
            SearchColor::ActiveBorder => "activeMatchBorder",
            SearchColor::ActiveOverview => "activeMatchColorOverviewRuler",
        };
        property(&self.0, key)
            .map(|value| value.as_string())
            .map_err(crate::js_error::callback_error)
    }
}
pub struct JsOptions(pub JsValue);
impl SearchOptionsSource for JsOptions {
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        let optional_bool = |key| {
            property(&self.0, key)
                .map(|value| (!value.is_undefined()).then(|| value.is_truthy()))
                .map_err(host_error)
        };
        let decorations = property(&self.0, "decorations").map_err(host_error)?;
        let decorations: Option<SearchDecorationOptionsHandle> = decorations
            .is_truthy()
            .then(|| Rc::new(JsDecorations(decorations)) as SearchDecorationOptionsHandle);
        Ok(SearchOptions {
            matching: xterm_core::search::SearchOptions {
                case_sensitive: optional_bool("caseSensitive")?,
                regex: optional_bool("regex")?,
                whole_word: optional_bool("wholeWord")?,
                // The upstream engine does not read incremental during a
                // normal search. In particular, do not invoke its JS getter.
                incremental: None,
                decorations: decorations.is_some(),
            },
            decorations,
        })
    }
}
pub fn term(value: JsValue) -> Result<Vec<u16>, JsValue> {
    if value.is_falsy() {
        return Ok(Vec::new());
    }
    let text = value
        .dyn_ref::<JsString>()
        .ok_or_else(|| js_sys::TypeError::new("Search term must be a string"))?;
    Ok(crate::utf16::from_js_string(text))
}
