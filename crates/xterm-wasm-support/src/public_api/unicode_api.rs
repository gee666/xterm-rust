// Copyright (c) 2019, 2021 The xterm.js authors. All rights reserved. MIT license.
//! A live Unicode view. Only obtaining the view is proposed-API gated.

use crate::{
    js_error::{callback_error, to_js_error},
    utf16::{from_js_string, to_js_string},
};
use js_sys::{Array, JsString};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use xterm_core::{
    event::EmitError,
    unicode::{UnicodeCharProperties, UnicodeCharWidth},
    unicode_callbacks::UnicodeProviderCallbacks,
    unicode_service::{UnicodeService, UnicodeServiceError},
};

#[wasm_bindgen(inline_js = "
export function unicodeVersion(provider) { return String(provider.version); }
export function unicodeWidth(provider, codepoint) { return provider.wcwidth(codepoint); }
export function unicodeProperties(provider, codepoint, preceding) {
  return provider.charProperties(codepoint, preceding) >>> 0;
}
export function unknownUnicodeVersion(version) {
  return new Error('unknown Unicode version \"' + version + '\"');
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = unicodeVersion)]
    fn version(provider: &JsValue) -> Result<JsString, JsValue>;
    #[wasm_bindgen(catch, js_name = unicodeWidth)]
    fn width(provider: &JsValue, codepoint: f64) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = unicodeProperties)]
    fn properties(provider: &JsValue, codepoint: u32, preceding: u32) -> Result<u32, JsValue>;
    #[wasm_bindgen(js_name = unknownUnicodeVersion)]
    fn unknown_version(version: &JsString) -> JsValue;
}

/// The portable registry uses Rust strings. Encode ill-formed UTF-16 and the
/// escape prefix itself, leaving ordinary names and array-index ordering intact.
/// This is a boundary representation, never a replacement-character conversion.
const PREFIX: &str = "\u{f0000}";

fn encode(value: &JsString) -> String {
    let units = from_js_string(value);
    if let Ok(text) = String::from_utf16(&units) {
        if !text.starts_with(PREFIX) {
            return text;
        }
    }
    let mut encoded = String::from(PREFIX);
    for unit in units {
        use std::fmt::Write;
        // Writing to a String cannot fail.
        write!(&mut encoded, "{unit:04x}").expect("String formatting is infallible");
    }
    encoded
}

fn decode(value: &str) -> JsString {
    if let Some(encoded) = value.strip_prefix(PREFIX) {
        let units: Option<Vec<u16>> = encoded
            .as_bytes()
            .chunks(4)
            .map(|chunk| {
                std::str::from_utf8(chunk).ok().and_then(|part| {
                    (part.len() == 4)
                        .then(|| u16::from_str_radix(part, 16).ok())
                        .flatten()
                })
            })
            .collect();
        if let Some(units) = units {
            return to_js_string(&units);
        }
    }
    JsString::from(value)
}

struct HostProvider(JsValue);

impl HostProvider {
    fn call_width(&self, codepoint: f64) -> Result<UnicodeCharWidth, EmitError> {
        let value = width(&self.0, codepoint).map_err(callback_error)?;
        match value.as_f64() {
            Some(0.0) => Ok(UnicodeCharWidth::Zero),
            Some(1.0) => Ok(UnicodeCharWidth::Narrow),
            Some(2.0) => Ok(UnicodeCharWidth::Wide),
            _ => Err(callback_error(
                js_sys::TypeError::new("Unicode wcwidth must return 0, 1 or 2").into(),
            )),
        }
    }
}

impl UnicodeProviderCallbacks for HostProvider {
    fn version(&self) -> Result<String, EmitError> {
        version(&self.0)
            .map(|value| encode(&value))
            .map_err(callback_error)
    }

    fn wcwidth(&self, codepoint: u32) -> Result<UnicodeCharWidth, EmitError> {
        self.call_width(f64::from(codepoint))
    }

    fn wcwidth_i32(&self, codepoint: i32) -> Result<UnicodeCharWidth, EmitError> {
        self.call_width(f64::from(codepoint))
    }

    fn char_properties(
        &self,
        codepoint: u32,
        preceding: UnicodeCharProperties,
    ) -> Result<UnicodeCharProperties, EmitError> {
        properties(&self.0, codepoint, preceding.raw())
            .map(UnicodeCharProperties::from_raw)
            .map_err(callback_error)
    }
}

/// Each public terminal getter returns a fresh view over the same Rust service.
#[wasm_bindgen(js_name = UnicodeApi)]
pub struct WebUnicodeApi {
    service: Rc<UnicodeService>,
}

impl WebUnicodeApi {
    pub fn new(service: Rc<UnicodeService>) -> Self {
        Self { service }
    }

    fn error(&self, error: &UnicodeServiceError) -> JsValue {
        match error {
            UnicodeServiceError::UnknownVersion(name) => unknown_version(&decode(name)),
            _ => to_js_error(error),
        }
    }
}

#[wasm_bindgen(js_class = UnicodeApi)]
impl WebUnicodeApi {
    pub fn register(&self, provider: JsValue) -> Result<(), JsValue> {
        self.service
            .register_callbacks(Rc::new(HostProvider(provider)))
            .map_err(|error| self.error(&error))
    }

    #[wasm_bindgen(getter)]
    pub fn versions(&self) -> Array {
        self.service
            .versions()
            .iter()
            .map(|value| JsValue::from(decode(value)))
            .collect()
    }

    #[wasm_bindgen(getter, js_name = activeVersion)]
    pub fn active_version(&self) -> JsString {
        decode(&self.service.active_version())
    }

    #[wasm_bindgen(setter, js_name = activeVersion)]
    pub fn set_active_version(&self, value: JsString) -> Result<(), JsValue> {
        self.service
            .set_active_version(&encode(&value))
            .map_err(|error| self.error(&error))
    }
}
