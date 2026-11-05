// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use xterm_addons::search::{
    SearchAddonError, SearchColor, SearchDecorationOptionsHandle, SearchDecorationSource,
    SearchHighlightLimit, SearchOptions, SearchOptionsHandle, SearchOptionsSource,
};

// These access/coercion operations need JS boxing, getter exceptions, BigInt and
// Symbol semantics. All matching, traversal, result state and decoration layout
// remain Rust. Static module imports also work under a no-eval CSP.
#[wasm_bindgen(
    inline_js = "export function searchAddonGet(v,k){return v?.[k];}\nexport function searchAddonFlag(v,k){return v[k];}\nexport function searchAddonRegex(v){return v.regex;}\nexport function searchAddonRefresh(v){return {...v,incremental:true};}\nexport function searchAddonLimitReached(v,n){return n>=v;}\nexport function searchAddonLimitEnd(v,n){let x=+v;if(Number.isNaN(x))x=0;x=Math.trunc(x);return x<0?Math.max(n+x,0):Math.min(x,n);}"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = searchAddonGet)]
    pub fn property(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = searchAddonFlag)]
    fn flag(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = searchAddonRegex)]
    fn regex(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = searchAddonRefresh)]
    fn refresh(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = searchAddonLimitReached)]
    fn reached(value: &JsValue, count: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = searchAddonLimitEnd)]
    fn end(value: &JsValue, length: f64) -> Result<f64, JsValue>;
}
pub(super) fn host_error(value: JsValue) -> SearchAddonError {
    SearchAddonError::Host(Rc::new(crate::js_error::JsCallbackError(value)))
}
pub fn no_scroll(value: &JsValue) -> Result<bool, SearchAddonError> {
    property(value, "noScroll")
        .map(|value| value.is_truthy())
        .map_err(host_error)
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
        flag(&self.0, key)
            .map(|value| value.as_string())
            .map_err(crate::js_error::callback_error)
    }
}
pub struct JsOptions(pub JsValue);
impl SearchOptionsSource for JsOptions {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn is_truthy(&self) -> bool {
        self.0.is_truthy()
    }
    fn for_refresh(&self) -> Result<SearchOptionsHandle, SearchAddonError> {
        refresh(&self.0)
            .map(|value| Rc::new(Self(value)) as SearchOptionsHandle)
            .map_err(host_error)
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        let value = property(&self.0, "decorations").map_err(host_error)?;
        Ok(value
            .is_truthy()
            .then(|| Rc::new(JsDecorations(value)) as SearchDecorationOptionsHandle))
    }
    fn highlight_decorations(&self) -> Result<SearchDecorationOptionsHandle, SearchAddonError> {
        flag(&self.0, "decorations")
            .map(|value| Rc::new(JsDecorations(value)) as SearchDecorationOptionsHandle)
            .map_err(host_error)
    }
    fn flags_changed(
        &self,
        stored: &dyn Fn() -> Option<SearchOptionsHandle>,
    ) -> Result<bool, SearchAddonError> {
        for (index, key) in ["caseSensitive", "regex", "wholeWord"]
            .into_iter()
            .enumerate()
        {
            // Capture this request before invoking its getter, but resolve the
            // next property from live addon state after both getters return.
            let current = stored();
            let left = match current.as_ref() {
                Some(options) => match options.as_any().downcast_ref::<Self>() {
                    Some(options) => flag(&options.0, key).map_err(host_error)?,
                    None => {
                        let flags = options.read()?.matching;
                        [flags.case_sensitive, flags.regex, flags.whole_word][index]
                            .map(JsValue::from_bool)
                            .unwrap_or(JsValue::UNDEFINED)
                    }
                },
                None => flag(&JsValue::UNDEFINED, key).map_err(host_error)?,
            };
            let right = flag(&self.0, key).map_err(host_error)?;
            if left != right {
                return Ok(true);
            }
        }
        Ok(false)
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        let decorations = self.decorations()?;
        let mut matching = self.matching_options()?;
        matching.decorations = decorations.is_some();
        Ok(SearchOptions {
            matching,
            decorations,
        })
    }
    fn matching_options(&self) -> Result<xterm_core::search::SearchOptions, SearchAddonError> {
        let optional_bool = |value: Result<JsValue, JsValue>| {
            value
                .map(|value| (!value.is_undefined()).then(|| value.is_truthy()))
                .map_err(host_error)
        };
        Ok(xterm_core::search::SearchOptions {
            // Unlike addon-level optional reads, engine matching requires
            // a non-null request and first accesses the regex property.
            // Keep a named property access so engine TypeErrors identify
            // regex in WebKit too, rather than an opaque computed key.
            regex: optional_bool(regex(&self.0))?,
            case_sensitive: optional_bool(flag(&self.0, "caseSensitive"))?,
            whole_word: optional_bool(flag(&self.0, "wholeWord"))?,
            // The upstream engine does not read incremental during a
            // normal search. In particular, do not invoke its JS getter.
            incremental: None,
            decorations: false,
        })
    }
}
