//! Retain a thrown JavaScript value through portable Rust callback errors.

use std::{error::Error, fmt};
use wasm_bindgen::JsValue;
use xterm_core::event::EmitError;

#[derive(Clone, Debug)]
pub struct JsCallbackError(pub JsValue);

impl fmt::Display for JsCallbackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Formatting an arbitrary thrown JS object could itself invoke a
        // throwing toString. Identity remains available in the stored value.
        f.write_str("JavaScript callback threw")
    }
}
impl Error for JsCallbackError {}

pub fn callback_error(value: JsValue) -> EmitError {
    EmitError::callback(JsCallbackError(value))
}

/// Recover the original JS exception through standard Rust error sources.
pub fn to_js_error(error: &(dyn Error + 'static)) -> JsValue {
    let mut current = Some(error);
    while let Some(cause) = current {
        if let Some(value) = cause.downcast_ref::<JsCallbackError>() {
            return value.0.clone();
        }
        if matches!(
            cause.downcast_ref::<xterm_core::buffer::BufferError>(),
            Some(xterm_core::buffer::BufferError::InvalidArrayLength)
        ) {
            return js_sys::RangeError::new(&cause.to_string()).into();
        }
        if matches!(
            cause.downcast_ref::<EmitError>(),
            Some(EmitError::ListenerArrayCleared)
        ) {
            return js_sys::TypeError::new(&cause.to_string()).into();
        }
        current = cause.source();
    }
    js_sys::Error::new(&error.to_string()).into()
}
