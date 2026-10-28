// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Shared extended cell attributes for raw reusable-cell destinations.
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::buffer::ExtendedAttrs;

const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

#[wasm_bindgen(js_name = ExtendedCellAttributes)]
pub struct WebExtendedAttrs {
    value: RefCell<ExtendedAttrs>,
}
impl WebExtendedAttrs {
    pub(super) fn new(value: ExtendedAttrs) -> Self {
        Self {
            value: RefCell::new(value),
        }
    }
}
#[wasm_bindgen]
impl WebExtendedAttrs {
    #[wasm_bindgen(getter)]
    pub fn ext(&self) -> u32 {
        self.value.borrow().ext()
    }
    #[wasm_bindgen(setter)]
    pub fn set_ext(&self, value: u32) {
        self.value.borrow_mut().set_ext(value);
    }
    #[wasm_bindgen(getter, js_name = urlId)]
    pub fn url_id(&self) -> f64 {
        self.value.borrow().url_id() as f64
    }
    #[wasm_bindgen(setter, js_name = urlId)]
    pub fn set_url_id(&self, value: f64) -> Result<(), JsValue> {
        if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value > MAX_SAFE_INTEGER {
            return Err(js_sys::RangeError::new(
                "Native hyperlink identifiers require a nonnegative safe integer",
            )
            .into());
        }
        self.value.borrow_mut().set_url_id(value as u64);
        Ok(())
    }
    #[wasm_bindgen(getter, js_name = underlineStyle)]
    pub fn underline_style(&self) -> u32 {
        self.value.borrow().underline_style()
    }
    #[wasm_bindgen(setter, js_name = underlineStyle)]
    pub fn set_underline_style(&self, value: u32) {
        self.value.borrow_mut().set_underline_style(value);
    }
    #[wasm_bindgen(getter, js_name = underlineColor)]
    pub fn underline_color(&self) -> u32 {
        self.value.borrow().underline_color()
    }
    #[wasm_bindgen(setter, js_name = underlineColor)]
    pub fn set_underline_color(&self, value: u32) {
        self.value.borrow_mut().set_underline_color(value);
    }
    #[wasm_bindgen(getter, js_name = underlineVariantOffset)]
    pub fn underline_variant_offset(&self) -> u32 {
        self.value.borrow().underline_variant_offset()
    }
    #[wasm_bindgen(setter, js_name = underlineVariantOffset)]
    pub fn set_underline_variant_offset(&self, value: u32) {
        self.value.borrow_mut().set_underline_variant_offset(value);
    }
    #[wasm_bindgen(getter)]
    pub fn payload(&self) -> Result<JsValue, JsValue> {
        let payload = self.value.borrow().payload();
        match payload {
            None => Ok(JsValue::UNDEFINED),
            Some(value) => value.downcast_ref::<JsValue>().cloned().ok_or_else(|| {
                js_sys::TypeError::new("Native attribute payload has no JavaScript representation")
                    .into()
            }),
        }
    }
    #[wasm_bindgen(setter)]
    pub fn set_payload(&self, value: JsValue) {
        self.value
            .borrow_mut()
            .set_payload(if value.is_undefined() {
                None
            } else {
                Some(Rc::new(value))
            });
    }
    #[wasm_bindgen(js_name = isEmpty)]
    pub fn is_empty(&self) -> bool {
        self.value.borrow().is_empty()
    }
    #[wasm_bindgen(js_name = clone)]
    pub fn clone_value(&self) -> Self {
        Self::new(self.value.borrow().clone())
    }
}
