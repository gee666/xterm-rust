// Copyright (c) 2024 The xterm.js authors. All rights reserved. MIT license.
use super::{coercion, event_error, event_function, subscription_event, SubscriptionEvent};
use crate::terminal::Terminal;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use xterm_addons::{
    native_status::HeadlessStatusTerminal,
    progress::{ProgressAddon as NativeAddon, ProgressState},
};
use xterm_core::lifecycle::Disposable;

#[derive(Default)]
struct BindingState {
    // The declared API uses numbers, but source stores the actual third state
    // property read, even an object or undefined supplied by an untyped caller.
    api_state: RefCell<Option<JsValue>>,
    api_event: Cell<bool>,
}
fn to_js(progress: &ProgressState, binding: &BindingState) -> Result<JsValue, JsValue> {
    let state = binding
        .api_state
        .borrow()
        .clone()
        .unwrap_or_else(|| progress.state.into());
    // Object literals create own data properties and must not invoke an
    // inherited state/value setter or fail on an inherited read-only field.
    coercion::progress_object(&state, progress.value)
}
#[wasm_bindgen]
pub struct ProgressAddon {
    addon: NativeAddon,
    binding: Rc<BindingState>,
    adapter: RefCell<Option<Rc<HeadlessStatusTerminal>>>,
    event: RefCell<Option<SubscriptionEvent>>,
}
#[wasm_bindgen]
impl ProgressAddon {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            addon: NativeAddon::new(),
            binding: Rc::new(BindingState::default()),
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
        let binding = self.binding.clone();
        event.subscribe(move |_| {
            // Protocol reports replace _st with a numeric state. API calls have
            // already stored their raw state before evaluating the value.
            if !binding.api_event.get() {
                binding.api_state.borrow_mut().take();
            }
        });
        let binding = self.binding.clone();
        *self.event.borrow_mut() = Some(subscription_event(event, move |progress| {
            to_js(progress, &binding)
        }));
        Ok(())
    }
    #[wasm_bindgen(getter)]
    pub fn progress(&self) -> Result<JsValue, JsValue> {
        to_js(&self.addon.progress(), &self.binding)
    }
    #[wasm_bindgen(setter)]
    pub fn set_progress(&self, progress: JsValue) -> Result<(), JsValue> {
        // Source reads state separately in each comparison and once again for
        // assignment. Getter calls and ToPrimitive failures are observable.
        if coercion::less(&coercion::property(&progress, "state")?, &0.into())?
            || coercion::less(&4.into(), &coercion::property(&progress, "state")?)?
        {
            web_sys::console::warn_1(&"progress state out of bounds, not applied".into());
            return Ok(());
        }
        let state = coercion::property(&progress, "state")?;
        *self.binding.api_state.borrow_mut() = Some(state);
        // Keep that partial assignment if the value getter/coercion throws.
        let value = coercion::number(&coercion::property(&progress, "value")?)?;
        // Value getters can reenter this setter or produce a protocol report.
        // Read the current state only after those callbacks, matching _st/_pr.
        let state = self.binding.api_state.borrow().as_ref().map_or_else(
            || self.addon.progress().state,
            |state| {
                state
                    .as_f64()
                    .filter(|n| !(*n < 0.0 || *n > 4.0))
                    .unwrap_or(f64::NAN)
            },
        );
        let previous = self.binding.api_event.replace(true);
        let result = self.addon.set_progress(ProgressState { state, value });
        self.binding.api_event.set(previous);
        if !result.map_err(event_error)? {
            return Err(
                js_sys::Error::new("native progress rejected a validated binding update").into(),
            );
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
