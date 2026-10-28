//! DOM callbacks owned by JavaScript's collector, with explicit removal tokens.
//! Fn callbacks support recursive dispatch; FnMut callbacks follow Rust's
//! non-reentrant mutable-closure contract. Prefer fallible Fn for terminal input.

use js_sys::Function;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, EventTarget};

pub struct DomListener {
    target: EventTarget,
    event_type: String,
    callback: Option<Function>,
    capture: bool,
    removal_failed: bool,
}
impl DomListener {
    pub fn new(target: &EventTarget, event_type: &str, capture: bool, callback: impl FnMut(Event) + 'static) -> Result<Self, JsValue> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>).into_js_value().unchecked_into();
        Self::register(target, event_type, capture, None, callback)
    }
    pub fn new_reentrant(target: &EventTarget, event_type: &str, capture: bool, callback: impl Fn(Event) + 'static) -> Result<Self, JsValue> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn Fn(Event)>).into_js_value().unchecked_into();
        Self::register(target, event_type, capture, None, callback)
    }
    pub fn new_with_options(target: &EventTarget, event_type: &str, capture: bool, passive: bool, callback: impl FnMut(Event) + 'static) -> Result<Self, JsValue> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>).into_js_value().unchecked_into();
        Self::register(target, event_type, capture, Some(passive), callback)
    }
    pub fn new_reentrant_with_options(target: &EventTarget, event_type: &str, capture: bool, passive: bool, callback: impl Fn(Event) + 'static) -> Result<Self, JsValue> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn Fn(Event)>).into_js_value().unchecked_into();
        Self::register(target, event_type, capture, Some(passive), callback)
    }
    pub fn new_reentrant_fallible(target: &EventTarget, event_type: &str, capture: bool, passive: Option<bool>, callback: impl Fn(Event) -> Result<(), JsValue> + 'static) -> Result<Self, JsValue> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn Fn(Event) -> Result<(), JsValue>>).into_js_value().unchecked_into();
        Self::register(target, event_type, capture, passive, callback)
    }
    fn register(target: &EventTarget, event_type: &str, capture: bool, passive: Option<bool>, callback: Function) -> Result<Self, JsValue> {
        // Transfer closure ownership before calling the host. A custom target
        // may register the function and then throw; its function remains valid.
        if let Some(passive) = passive {
            let options = web_sys::AddEventListenerOptions::new();
            options.set_capture(capture);
            options.set_passive(passive);
            target.add_event_listener_with_callback_and_add_event_listener_options(event_type, &callback, &options)?;
        } else {
            target.add_event_listener_with_callback_and_bool(event_type, &callback, capture)?;
        }
        Ok(Self { target: target.clone(), event_type: event_type.into(), callback: Some(callback), capture, removal_failed: false })
    }
    pub fn dispose(&mut self) -> Result<(), JsValue> {
        if let Some(callback) = self.callback.as_ref() {
            if let Err(error) = self.target.remove_event_listener_with_callback_and_bool(&self.event_type, callback, self.capture) {
                self.removal_failed = true;
                return Err(error);
            }
            self.callback.take();
            self.removal_failed = false;
        }
        Ok(())
    }
    pub fn is_disposed(&self) -> bool { self.callback.is_none() }
}
impl Drop for DomListener {
    fn drop(&mut self) {
        // A failed explicit removal has already returned the original error.
        // Do not retry and panic during unwinding. Any still-registered callback
        // remains JS-owned and valid, including across origin/window changes.
        if !self.removal_failed {
            if let Err(error) = self.dispose() { crate::write_scheduler::report_unhandled_error(&error); }
        }
    }
}
