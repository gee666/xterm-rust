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
    fn descriptors(
        value: &JsValue,
        link: &JsValue,
        pointer_get: &JsValue,
        pointer_set: &JsValue,
        underline_get: &JsValue,
        underline_set: &JsValue,
    ) -> Result<(), JsValue>;
}
impl State {
    pub(super) fn set_decoration(
        &self,
        underline: bool,
        value: JsValue,
        link: &BrowserLink,
    ) -> Result<(), JsValue> {
        let current = self.hover.borrow().current().cloned();
        let Some(current) = current else {
            return Ok(());
        };
        let old = {
            let mut values = current.decoration_values.borrow_mut();
            let Some(values) = values.as_mut() else {
                return Ok(());
            };
            let field = if underline {
                &mut values.0
            } else {
                &mut values.1
            };
            std::mem::replace(field, value.clone())
        };
        if old == value {
            return Ok(());
        }
        let hovered = current.hovered.get();
        if hovered {
            if underline {
                self.fire_underline(link, value.is_truthy())?;
            } else {
                self.element
                    .class_list()
                    .toggle_with_force("xterm-cursor-pointer", value.is_truthy())?;
            }
        }
        Ok(())
    }
    pub(super) fn install_decorations(self: &Rc<Self>, link: &BrowserLink) -> Result<(), JsValue> {
        let BrowserLink::JavaScript(link) = link else {
            return Ok(());
        };
        set(link, "decorations", &Object::new())?;
        let getter = |underline| {
            let weak = Rc::downgrade(self);
            Closure::<dyn Fn() -> JsValue>::new(move || {
                let Some(state) = weak.upgrade() else {
                    return JsValue::UNDEFINED;
                };
                let current = state.hover.borrow().current().cloned();
                let Some(current) = current else {
                    return JsValue::UNDEFINED;
                };
                let values = current.decoration_values.borrow();
                let Some(values) = values.as_ref() else {
                    return JsValue::UNDEFINED;
                };
                if underline {
                    values.0.clone()
                } else {
                    values.1.clone()
                }
            })
            .into_js_value()
        };
        let setter = |underline| {
            let weak = Rc::downgrade(self);
            Closure::<dyn Fn(JsValue, JsValue) -> Result<(), JsValue>>::new(move |link, value| {
                if let Some(state) = weak.upgrade() {
                    state.set_decoration(underline, value, &BrowserLink::JavaScript(link))?;
                }
                Ok(())
            })
            .into_js_value()
        };
        descriptors(
            &property(link, "decorations")?,
            link,
            &getter(false),
            &setter(false),
            &getter(true),
            &setter(true),
        )
    }
}
