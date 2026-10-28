//! Lossless option values between the portable service and JavaScript.
use crate::{
    js_error::{callback_error, to_js_error},
    utf16::{from_js_string, to_js_string},
};
use js_sys::{JsString, Object};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::prelude::*;
use xterm_core::options::{
    CoercionError, CoercionHint, NativeProperties, NumericSemantics, OptionError, OptionObject,
    OptionValue as V, PropertyAccess, WindowsPty,
};

#[wasm_bindgen(inline_js = "
export function optionData(target, key, value) { Object.defineProperty(target,key,{value,writable:true,enumerable:true,configurable:true}); }
export function optionNumber(value) { return +value; }
export function optionLess(value, rhs) { return value < rhs; }
export function optionLessEqual(value, rhs) { return value <= rhs; }
export function optionMultiply(value, rhs) { return value * rhs; }
export function optionString(value) { return `${value}`; }
export function optionConcat(value) { return '' + value; }
export function optionAdd(left, right) { return left + right; }
export function optionTruthy(value) { return !!value; }
export function optionProperty(value, key) { return value[key]; }
export function optionSetProperty(value,key,data) { value[key]=data; }
export function optionDeleteProperty(value,key) { return delete value[key]; }
export function optionKeys(value) { return Object.keys(value); }
export function optionError(message, type) { return type ? new TypeError(message) : new Error(message); }
")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn optionData(target: &Object, key: &JsString, value: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn optionNumber(value: &JsValue) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionLess(value: &JsValue, rhs: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionLessEqual(value: &JsValue, rhs: f64) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionMultiply(value: &JsValue, rhs: f64) -> Result<f64, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionString(value: &JsValue) -> Result<JsString, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionConcat(value: &JsValue) -> Result<JsString, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionAdd(left: &JsValue, right: &JsValue) -> Result<JsValue, JsValue>;
    fn optionTruthy(value: &JsValue) -> bool;
    #[wasm_bindgen(catch)]
    fn optionProperty(value: &JsValue, key: &JsString) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionSetProperty(value: &JsValue, key: &JsString, data: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch)]
    fn optionDeleteProperty(value: &JsValue, key: &JsString) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch)]
    fn optionKeys(value: &JsValue) -> Result<js_sys::Array, JsValue>;
    fn optionError(message: &JsString, type_error: bool) -> JsValue;
}

pub fn set_data(target: &Object, key: &str, value: &JsValue) -> Result<(), JsValue> {
    optionData(target, &JsString::from(key), value)
}

pub fn error_to_js(error: &OptionError) -> JsValue {
    match error {
        OptionError::Invalid(_) | OptionError::UnknownKey(_) => {
            optionError(&to_js_string(&error.message_utf16()), false)
        }
        OptionError::Coercion(_) => optionError(&to_js_string(&error.message_utf16()), true),
        OptionError::Dispatch(_) => to_js_error(error),
    }
}

struct JsNumeric(JsValue);
/// Reuse exact host operators for a typed adapter with the same backing value.
pub fn operator_semantics(value: &JsValue) -> Rc<dyn NumericSemantics> {
    Rc::new(JsNumeric(value.clone()))
}
impl NumericSemantics for JsNumeric {
    fn add_left(&self, rhs: &V) -> Option<Result<V, OptionError>> {
        Some(
            into_js(rhs)
                .and_then(|rhs| optionAdd(&self.0, &rhs))
                .and_then(|result| from_js(&result))
                .map_err(|error| OptionError::Dispatch(callback_error(error))),
        )
    }
    fn add_right(&self, lhs: &V) -> Option<Result<V, OptionError>> {
        Some(
            into_js(lhs)
                .and_then(|lhs| optionAdd(&lhs, &self.0))
                .and_then(|result| from_js(&result))
                .map_err(|error| OptionError::Dispatch(callback_error(error))),
        )
    }
    fn concat_utf16(&self) -> Option<Result<Vec<u16>, OptionError>> {
        Some(
            optionConcat(&self.0)
                .map(|value| from_js_string(&value))
                .map_err(|error| OptionError::Dispatch(callback_error(error))),
        )
    }
    fn less_than(&self, rhs: f64) -> Result<bool, OptionError> {
        optionLess(&self.0, rhs).map_err(|e| OptionError::Dispatch(callback_error(e)))
    }
    fn less_equal(&self, rhs: f64) -> Result<bool, OptionError> {
        optionLessEqual(&self.0, rhs).map_err(|e| OptionError::Dispatch(callback_error(e)))
    }
    fn multiply(&self, rhs: f64) -> Result<f64, OptionError> {
        optionMultiply(&self.0, rhs).map_err(|e| OptionError::Dispatch(callback_error(e)))
    }
}

struct JsProperties(JsValue);
impl PropertyAccess for JsProperties {
    fn get(&self, key: &str) -> Result<V, OptionError> {
        optionProperty(&self.0, &JsString::from(key))
            .and_then(|value| from_js(&value))
            .map_err(|e| OptionError::Dispatch(callback_error(e)))
    }
    fn set(&self, key: &str, value: V) -> Result<(), OptionError> {
        let value = into_js(&value).map_err(|e| OptionError::Dispatch(callback_error(e)))?;
        optionSetProperty(&self.0, &JsString::from(key), &value)
            .map_err(|e| OptionError::Dispatch(callback_error(e)))
    }
    fn delete(&self, key: &str) -> Result<bool, OptionError> {
        optionDeleteProperty(&self.0, &JsString::from(key))
            .map_err(|e| OptionError::Dispatch(callback_error(e)))
    }
    fn keys(&self) -> Result<Vec<String>, OptionError> {
        optionKeys(&self.0)
            .map_err(|e| OptionError::Dispatch(callback_error(e)))?
            .iter()
            .map(|key| {
                key.as_string().ok_or_else(|| {
                    CoercionError::new("host property enumeration returned a non-string key").into()
                })
            })
            .collect()
    }
}

pub fn property_access(value: &JsValue) -> Rc<dyn PropertyAccess> {
    Rc::new(JsProperties(value.clone()))
}

pub fn from_js(value: &JsValue) -> Result<V, JsValue> {
    if value.is_undefined() {
        return Ok(V::Undefined);
    }
    if value.is_null() {
        return Ok(V::Null);
    }
    if let Some(value) = value.as_bool() {
        return Ok(V::Bool(value));
    }
    if let Some(value) = value.as_f64() {
        return Ok(V::Number(value));
    }
    if value.is_string() {
        return Ok(V::String(from_js_string(&JsString::from(value.clone()))));
    }
    let payload = value.clone();
    let primitive = payload.clone();
    // Symbol/BigInt stay opaque host values. Unary plus and template conversion
    // preserve the source's TypeErrors; truthiness/equality retain JS semantics.
    let mut object = OptionObject::with_numeric_value_semantics(
        payload,
        move |hint| {
            match hint {
                CoercionHint::Number => optionNumber(&primitive).map(V::Number),
                CoercionHint::String => {
                    optionString(&primitive).map(|s| V::String(from_js_string(&s)))
                }
            }
            .map_err(|error| OptionError::Dispatch(callback_error(error)))
        },
        |payload, key| {
            optionProperty(payload, &JsString::from(key))
                .and_then(|value| from_js(&value))
                .map_err(|error| OptionError::Dispatch(callback_error(error)))
        },
        optionTruthy(value),
        |a, b| a == b,
        operator_semantics(value),
    );
    OptionObject::attach_property_access(&mut object, property_access(value))
        .map_err(|error| error_to_js(&error))?;
    Ok(V::Object(object))
}

#[derive(Default)]
pub struct ConversionCache(RefCell<Vec<(Weak<OptionObject>, JsValue)>>);
impl ConversionCache {
    pub fn remember(&self, value: &V, host: &JsValue) {
        if let V::Object(object) = value {
            let key = Rc::downgrade(object);
            let mut entries = self.0.borrow_mut();
            entries.retain(|(existing, _)| {
                existing.strong_count() > 0 && !Weak::ptr_eq(existing, &key)
            });
            entries.push((key, host.clone()));
        }
    }

    pub fn convert(&self, value: &V) -> Result<JsValue, JsValue> {
        let V::Object(object) = value else {
            return into_js(value);
        };
        if let Some(value) = object
            .backing::<JsValue>()
            .or_else(|| object.payload::<JsValue>())
        {
            return Ok(value.clone());
        }
        self.0
            .borrow_mut()
            .retain(|(key, _)| key.strong_count() > 0);
        if let Some(value) = self.0.borrow().iter().find_map(|(key, value)| {
            key.upgrade()
                .filter(|key| Rc::ptr_eq(key, object))
                .map(|_| value.clone())
        }) {
            return Ok(value);
        }
        let value = into_js(value)?;
        self.0
            .borrow_mut()
            .push((Rc::downgrade(object), value.clone()));
        Ok(value)
    }
}

pub fn into_js(value: &V) -> Result<JsValue, JsValue> {
    Ok(match value {
        V::Undefined => JsValue::UNDEFINED,
        V::Null => JsValue::NULL,
        V::Bool(value) => JsValue::from_bool(*value),
        V::Number(value) => JsValue::from_f64(*value),
        V::String(value) => to_js_string(value).into(),
        V::Object(value) => {
            if let Some(value) = value
                .backing::<JsValue>()
                .or_else(|| value.payload::<JsValue>())
            {
                return Ok(value.clone());
            }
            let object = Object::new();
            if let Some(entries) = value.payload::<Rc<NativeProperties>>() {
                for (key, value) in entries.snapshot() {
                    set_data(&object, &key, &into_js(&value)?)?;
                }
            } else if let Some(pty) = value.payload::<WindowsPty>() {
                if let Some(backend) = &pty.backend {
                    set_data(&object, "backend", &backend.as_str().into())?;
                }
                if let Some(build) = pty.build_number {
                    set_data(&object, "buildNumber", &build.into())?;
                }
            } else if value.payload::<()>().is_none() {
                return Err(js_sys::TypeError::new(
                    "Native option payload has no JavaScript representation",
                )
                .into());
            }
            object.into()
        }
    })
}
