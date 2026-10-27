//! Removable DOM callbacks. Ownership never uses Closure::forget.
//!
//! Explicit disposal propagates host exceptions. Drop is a last-resort cleanup;
//! built-in removeEventListener does not throw for a valid EventTarget.

use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Event, EventTarget};

pub struct DomListener {
    target: EventTarget,
    event_type: String,
    callback: Option<Closure<dyn FnMut(Event)>>,
    capture: bool,
}

impl DomListener {
    pub fn new(
        target: &EventTarget,
        event_type: &str,
        capture: bool,
        callback: impl FnMut(Event) + 'static,
    ) -> Result<Self, JsValue> {
        let callback = Closure::wrap(Box::new(callback) as Box<dyn FnMut(Event)>);
        target.add_event_listener_with_callback_and_bool(
            event_type,
            callback.as_ref().unchecked_ref(),
            capture,
        )?;
        Ok(Self {
            target: target.clone(),
            event_type: event_type.into(),
            callback: Some(callback),
            capture,
        })
    }

    pub fn dispose(&mut self) -> Result<(), JsValue> {
        if let Some(callback) = self.callback.as_ref() {
            self.target.remove_event_listener_with_callback_and_bool(
                &self.event_type,
                callback.as_ref().unchecked_ref(),
                self.capture,
            )?;
            self.callback.take();
        }
        Ok(())
    }

    pub fn is_disposed(&self) -> bool {
        self.callback.is_none()
    }
}

impl Drop for DomListener {
    fn drop(&mut self) {
        // EventTarget/removeEventListener is a built-in non-throwing DOM
        // operation with a previously validated callback/type/capture tuple.
        self.dispose()
            .expect("built-in removeEventListener cannot throw for this listener");
    }
}
