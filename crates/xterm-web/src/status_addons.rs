//! Browser bindings for Rust status addon logic. No JavaScript terminal engine
//! parses the OSC reports or maintains addon state.

mod coercion;
mod program;
mod progress;
pub use program::ProgramStatusAddon;
pub use progress::ProgressAddon;

use js_sys::{Function, Object, Reflect};
use wasm_bindgen::prelude::*;
use xterm_core::event::{EmitError, Event};

pub(crate) fn event_error(error: EmitError) -> JsValue {
    crate::js_error::to_js_error(&error)
}
type SubscriptionEvent = Function;
fn subscription_event<T: 'static>(
    event: Event<T>,
    convert: impl Fn(&T) -> Result<JsValue, JsValue> + 'static,
) -> SubscriptionEvent {
    // One shared event binding implementation preserves subscription disposal,
    // exact JS exceptions and JS-owned registration function lifetime.
    crate::terminal::events::callable(event, convert)
}
fn event_function(event: &SubscriptionEvent) -> JsValue {
    event.clone().into()
}
fn set(object: &Object, key: &str, value: &JsValue) -> Result<(), JsValue> {
    if Reflect::set(object, &key.into(), value)? {
        Ok(())
    } else {
        Err(js_sys::TypeError::new("status record field is not writable").into())
    }
}
