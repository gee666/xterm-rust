// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
//! JS owns descriptors and link references. Rust closures retain only weak state.
use super::{BrowserLink, State};
use crate::links::browser_payload::{property, set};
use js_sys::Object;
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, prelude::*};

#[wasm_bindgen(module = "/src/links/raw_link_bridge.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = linkDescriptors)]
    fn descriptors(value: &JsValue, link: &JsValue, pointer_get: &JsValue, pointer_set: &JsValue, underline_get: &JsValue, underline_set: &JsValue) -> Result<(), JsValue>;
}
impl State {
    pub(super) fn set_decoration(&self, underline: bool, value: JsValue, link: &BrowserLink) -> Result<(), JsValue> {
        if self.hover.borrow().current().is_none() { return Ok(()); }
        let old = {
            let mut values = self.decoration_values.borrow_mut();
            let field = if underline { &mut values.0 } else { &mut values.1 };
            std::mem::replace(field, value.clone())
        };
        if old == value { return Ok(()); }
        let hovered = self.hover.borrow().is_hovered();
        if hovered {
            if underline { self.fire_underline(link, value.is_truthy())?; }
            else { self.element.class_list().toggle_with_force("xterm-cursor-pointer", value.is_truthy())?; }
        }
        Ok(())
    }
    pub(super) fn install_decorations(self: &Rc<Self>, link: &BrowserLink) -> Result<(), JsValue> {
        let BrowserLink::JavaScript(link) = link else { return Ok(()) };
        set(link, "decorations", &Object::new())?;
        let getter = |underline| {
            let weak = Rc::downgrade(self);
            Closure::<dyn Fn() -> JsValue>::new(move || {
                let Some(state) = weak.upgrade() else { return JsValue::UNDEFINED };
                if state.hover.borrow().current().is_none() { return JsValue::UNDEFINED; }
                let values = state.decoration_values.borrow();
                if underline { values.0.clone() } else { values.1.clone() }
            }).into_js_value()
        };
        let setter = |underline| {
            let weak = Rc::downgrade(self);
            Closure::<dyn Fn(JsValue, JsValue) -> Result<(), JsValue>>::new(move |link, value| {
                if let Some(state) = weak.upgrade() {
                    state.set_decoration(underline, value, &BrowserLink::JavaScript(link))?;
                }
                Ok(())
            }).into_js_value()
        };
        descriptors(&property(link, "decorations")?, link, &getter(false), &setter(false), &getter(true), &setter(true))
    }
}
