// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of browser/LocalizableStrings.ts.
use std::{cell::RefCell, collections::TryReserveError, fmt, rc::Rc};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

#[derive(Debug)]
pub enum LocalizationError {
    NotString,
    Allocation(TryReserveError),
}
impl fmt::Display for LocalizationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotString => f.write_str("localized value is not a string"),
            Self::Allocation(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for LocalizationError {}
impl From<TryReserveError> for LocalizationError {
    fn from(error: TryReserveError) -> Self {
        Self::Allocation(error)
    }
}
#[derive(Clone, Debug)]
enum StoredValue {
    Text(Rc<Vec<u16>>),
    #[cfg(target_arch = "wasm32")]
    Raw(JsValue),
}
#[derive(Debug)]
struct Strings {
    prompt: StoredValue,
    overflow: StoredValue,
}
impl Default for Strings {
    fn default() -> Self {
        Self {
            prompt: StoredValue::Text(Rc::new("Terminal input".encode_utf16().collect())),
            overflow: StoredValue::Text(Rc::new(
                "Too much output to announce, navigate to rows manually to read"
                    .encode_utf16()
                    .collect(),
            )),
        }
    }
}

/// Shared by terminals in a binding runtime, including different documents.
/// Native callers use UTF-16 text. Browser bindings retain unvalidated JS values
/// without coercing on assignment, as upstream does. DOM use performs coercion.
#[derive(Clone, Default, Debug)]
pub struct LocalizableStrings {
    values: Rc<RefCell<Strings>>,
}
impl LocalizableStrings {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn prompt_label(&self) -> Result<Vec<u16>, LocalizationError> {
        self.text(true)
    }
    pub fn too_much_output(&self) -> Result<Vec<u16>, LocalizationError> {
        self.text(false)
    }
    pub fn set_prompt_label(&self, value: Vec<u16>) {
        self.values.borrow_mut().prompt = StoredValue::Text(Rc::new(value));
    }
    pub fn set_too_much_output(&self, value: Vec<u16>) {
        self.values.borrow_mut().overflow = StoredValue::Text(Rc::new(value));
    }
    fn snapshot(&self, prompt: bool) -> StoredValue {
        let values = self.values.borrow();
        if prompt {
            values.prompt.clone()
        } else {
            values.overflow.clone()
        }
    }
    fn text(&self, prompt: bool) -> Result<Vec<u16>, LocalizationError> {
        match self.snapshot(prompt) {
            StoredValue::Text(text) => {
                let mut result = Vec::new();
                result.try_reserve(text.len())?;
                result.extend_from_slice(&text);
                Ok(result)
            }
            #[cfg(target_arch = "wasm32")]
            StoredValue::Raw(raw) => {
                if !raw.is_string() {
                    return Err(LocalizationError::NotString);
                }
                let text: &js_sys::JsString = raw.unchecked_ref();
                let mut result = Vec::new();
                result.try_reserve(text.length() as usize)?;
                result.extend(text.iter());
                Ok(result)
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    pub fn prompt_label_value(&self) -> JsValue {
        self.value(true)
    }
    #[cfg(target_arch = "wasm32")]
    pub fn too_much_output_value(&self) -> JsValue {
        self.value(false)
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_prompt_label_value(&self, value: JsValue) {
        self.values.borrow_mut().prompt = StoredValue::Raw(value);
    }
    #[cfg(target_arch = "wasm32")]
    pub fn set_too_much_output_value(&self, value: JsValue) {
        self.values.borrow_mut().overflow = StoredValue::Raw(value);
    }
    #[cfg(target_arch = "wasm32")]
    fn value(&self, prompt: bool) -> JsValue {
        match self.snapshot(prompt) {
            StoredValue::Text(text) => crate::utf16::to_js_string(&text).into(),
            StoredValue::Raw(value) => value,
        }
    }
}
