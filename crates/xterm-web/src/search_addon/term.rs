// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use js_sys::JsString;
use std::{any::Any, borrow::Cow};
use wasm_bindgen::{prelude::*, JsCast};
use xterm_addons::search::{SearchAddonError, SearchTermSource};

// Property access and relational conversion must use JS semantics, including
// length getters, Symbol failures and reentry. Matching remains in Rust.
#[wasm_bindgen(
    inline_js = "export function searchAddonValidTerm(v){return !!(v && v.length > 0);}"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = searchAddonValidTerm)]
    fn valid(value: &JsValue) -> Result<bool, JsValue>;
}
pub struct JsTerm(pub JsValue);
impl SearchTermSource for JsTerm {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        valid(&self.0).map_err(super::options::host_error)
    }
    fn is_truthy(&self) -> bool {
        self.0.is_truthy()
    }
    fn is_undefined(&self) -> bool {
        self.0.is_undefined()
    }
    fn equals(&self, other: &dyn SearchTermSource) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.0 == other.0)
    }
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError> {
        let text = self.0.dyn_ref::<JsString>().ok_or_else(|| {
            super::options::host_error(
                js_sys::TypeError::new("Search term must be a string").into(),
            )
        })?;
        Ok(Cow::Owned(crate::utf16::from_js_string(text)))
    }
}
