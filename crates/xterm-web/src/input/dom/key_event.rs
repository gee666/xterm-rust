// Licensed under MIT. DOM fields are read only when the Rust policy asks.
use crate::{
    input::keyboard_dispatch::{EventNumber, KeyEventSource},
    utf16::from_js_string,
};
use std::borrow::Cow;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::KeyboardEvent;
use xterm_core::input::keyboard_event_source::KeyboardEventSource;

pub(super) struct LiveKeyEvent<'a>(pub(super) &'a KeyboardEvent);
impl LiveKeyEvent<'_> {
    fn get(&self, name: &str) -> Result<JsValue, JsValue> {
        js_sys::Reflect::get(self.0.as_ref(), &JsValue::from_str(name))
    }
    fn string(&self, name: &str) -> Result<Cow<'_, [u16]>, JsValue> {
        let value = self.get(name)?;
        let string = value.dyn_into::<js_sys::JsString>().map_err(|_| {
            js_sys::TypeError::new(&format!("KeyboardEvent.{name} must be a string"))
        })?;
        Ok(Cow::Owned(from_js_string(&string)))
    }
    fn optional_number(&self, name: &str) -> Result<EventNumber, JsValue> {
        let value = self.get(name)?;
        if value.is_undefined() {
            return Ok(EventNumber::Undefined);
        }
        if value.is_null() {
            return Ok(EventNumber::Null);
        }
        value.as_f64().map(EventNumber::Number).ok_or_else(|| {
            js_sys::TypeError::new(&format!("KeyboardEvent.{name} must be a number")).into()
        })
    }
}
impl KeyboardEventSource for LiveKeyEvent<'_> {
    type Error = JsValue;
    fn key_code(&self) -> Result<f64, JsValue> {
        // Missing/null synthetic key codes cannot match a numeric switch case.
        // Preserve f64 values before the source's own arithmetic/conversions.
        Ok(match self.optional_number("keyCode")? {
            EventNumber::Number(number) => number,
            _ => f64::NAN,
        })
    }
    fn key(&self) -> Result<Cow<'_, [u16]>, JsValue> {
        self.string("key")
    }
    fn code(&self) -> Result<Cow<'_, [u16]>, JsValue> {
        self.string("code")
    }
    fn shift_key(&self) -> Result<bool, JsValue> {
        Ok(self.get("shiftKey")?.is_truthy())
    }
    fn alt_key(&self) -> Result<bool, JsValue> {
        Ok(self.get("altKey")?.is_truthy())
    }
    fn ctrl_key(&self) -> Result<bool, JsValue> {
        Ok(self.get("ctrlKey")?.is_truthy())
    }
    fn meta_key(&self) -> Result<bool, JsValue> {
        Ok(self.get("metaKey")?.is_truthy())
    }
}
impl KeyEventSource for LiveKeyEvent<'_> {
    fn repeat(&self) -> Result<bool, JsValue> {
        Ok(self.get("repeat")?.is_truthy())
    }
    fn char_code(&self) -> Result<EventNumber, JsValue> {
        self.optional_number("charCode")
    }
    fn which(&self) -> Result<EventNumber, JsValue> {
        self.optional_number("which")
    }
    fn is_key_press(&self) -> Result<bool, JsValue> {
        Ok(self.get("type")? == JsValue::from_str("keypress"))
    }
    fn alt_graph(&self) -> Result<bool, JsValue> {
        let method = self
            .get("getModifierState")?
            .dyn_into::<js_sys::Function>()
            .map_err(|_| {
                js_sys::TypeError::new("KeyboardEvent.getModifierState is not a function")
            })?;
        Ok(method
            .call1(self.0.as_ref(), &JsValue::from_str("AltGraph"))?
            .is_truthy())
    }
}
