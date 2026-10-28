// Copyright (c) 2024 The xterm.js authors. All rights reserved. MIT license.
use super::{event_error, event_function, set, subscription_event, SubscriptionEvent};
use crate::terminal::Terminal;
use js_sys::{Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_addons::{
    native_status::HeadlessStatusTerminal,
    progress::{ProgressAddon as NativeAddon, ProgressState},
};
use xterm_core::lifecycle::Disposable;

fn to_js(progress: &ProgressState) -> Result<JsValue, JsValue> {
    let object = Object::new();
    set(&object, "state", &progress.state.into())?;
    set(&object, "value", &progress.value.into())?;
    Ok(object.into())
}
fn number(value: &JsValue, key: &str) -> Result<f64, JsValue> {
    Reflect::get(value, &key.into())?
        .as_f64()
        .ok_or_else(|| js_sys::TypeError::new(&format!("progress {key} must be a number")).into())
}
#[wasm_bindgen]
pub struct ProgressAddon {
    addon: NativeAddon,
    adapter: RefCell<Option<Rc<HeadlessStatusTerminal>>>,
    event: RefCell<Option<SubscriptionEvent>>,
}
#[wasm_bindgen]
impl ProgressAddon {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            addon: NativeAddon::new(),
            adapter: RefCell::new(None),
            event: RefCell::new(None),
        }
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let adapter = Rc::new(
            HeadlessStatusTerminal::from_terminal(terminal.shared_native())
                .map_err(|e| js_sys::Error::new(&e.to_string()))?,
        );
        self.addon.activate(adapter.clone());
        *self.adapter.borrow_mut() = Some(adapter);
        let event = self
            .addon
            .on_change()
            .ok_or_else(|| js_sys::Error::new("progress emitter was not initialized"))?;
        *self.event.borrow_mut() = Some(subscription_event(event, to_js));
        Ok(())
    }
    #[wasm_bindgen(getter)]
    pub fn progress(&self) -> Result<JsValue, JsValue> {
        to_js(&self.addon.progress())
    }
    #[wasm_bindgen(setter)]
    pub fn set_progress(&self, progress: JsValue) -> Result<(), JsValue> {
        let state = ProgressState {
            state: number(&progress, "state")?,
            value: number(&progress, "value")?,
        };
        if !self.addon.set_progress(state).map_err(event_error)? {
            web_sys::console::warn_1(&"progress state out of bounds, not applied".into());
        }
        Ok(())
    }
    #[wasm_bindgen(getter,js_name=onChange)]
    pub fn on_change(&self) -> JsValue {
        self.event
            .borrow()
            .as_ref()
            .map_or(JsValue::UNDEFINED, event_function)
    }
    pub fn dispose(&self) {
        self.addon.dispose();
    }
}
impl Default for ProgressAddon {
    fn default() -> Self {
        Self::new()
    }
}
