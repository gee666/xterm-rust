// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Adapted from common/services/OptionsService.ts.

//! Options at the terminal's JS-compatible boundary. Values deliberately retain
//! upstream permissiveness instead of silently rejecting values that JS accepts.
//! Engine consumers should validate the type they require before using a value.
//! Platform detection is supplied by adapters through `is_mac`.
mod defaults;
mod validation;
mod value;

use crate::event::{EmitError, Emitter, Event};
use crate::lifecycle::{Disposable, DisposableHandle};
pub use defaults::default_options;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
pub use value::{CoercionError, CoercionHint, OptionObject, OptionValue};

/// Typed payload for the windowsPty object at the native/browser adapter boundary.
/// Absence differs from zero; consumers preserve upstream truthiness checks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WindowsPty {
    pub backend: Option<String>,
    pub build_number: Option<f64>,
}

impl From<WindowsPty> for OptionValue {
    fn from(value: WindowsPty) -> Self {
        Self::Object(OptionObject::with_properties(
            value,
            |_| Ok(Self::from("[object Object]")),
            |pty, key| {
                Ok(match key {
                    "backend" => pty
                        .backend
                        .as_deref()
                        .map(Self::from)
                        .unwrap_or(Self::Undefined),
                    "buildNumber" => pty
                        .build_number
                        .map(Self::Number)
                        .unwrap_or(Self::Undefined),
                    _ => Self::Undefined,
                })
            },
        ))
    }
}

type Values = Vec<(&'static str, OptionValue)>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptionError {
    UnknownKey(String),
    Invalid(Vec<u16>),
    Coercion(CoercionError),
    Dispatch(EmitError),
}
impl fmt::Display for OptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownKey(key) => write!(f, "No option with key \"{key}\""),
            Self::Invalid(message) => f.write_str(&String::from_utf16_lossy(message)),
            Self::Dispatch(error) => error.fmt(f),
            Self::Coercion(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for OptionError {}

impl OptionError {
    /// Lossless host error text, including JavaScript lone surrogates. Display
    /// replaces unpaired surrogates because Rust's formatter requires UTF-8.
    pub fn message_utf16(&self) -> Vec<u16> {
        match self {
            Self::Invalid(message) => message.clone(),
            Self::Coercion(error) => error.0.clone(),
            _ => self.to_string().encode_utf16().collect(),
        }
    }

    pub(super) fn invalid(prefix: &str, value: &OptionValue, suffix: &str) -> Result<Self, Self> {
        let mut message: Vec<u16> = prefix.encode_utf16().collect();
        message.extend(value.text_utf16()?);
        message.extend(suffix.encode_utf16());
        Ok(Self::Invalid(message))
    }
}
impl From<CoercionError> for OptionError {
    fn from(error: CoercionError) -> Self {
        Self::Coercion(error)
    }
}

/// Shared interior mutability permits synchronous listeners to read or update
/// options. No values borrow is held during callbacks. Constructor diagnostics
/// are returned rather than being written to the host's console.
pub struct OptionsService {
    values: Rc<RefCell<Values>>,
    changes: Emitter<&'static str>,
    diagnostics: Vec<OptionError>,
    dispatch_errors: Rc<RefCell<Vec<Option<OptionError>>>>,
}

// A frame per fire preserves nested/caught callback errors. Drop also removes
// the frame if a Rust callback unwinds, without catching or hiding that panic.
struct DispatchFrame<'a>(&'a RefCell<Vec<Option<OptionError>>>);
impl Drop for DispatchFrame<'_> {
    fn drop(&mut self) {
        self.0.borrow_mut().pop();
    }
}

impl OptionsService {
    pub fn new<K: AsRef<str>>(
        is_mac: bool,
        options: impl IntoIterator<Item = (K, OptionValue)>,
    ) -> Self {
        let mut values = default_options(is_mac);
        let mut diagnostics = Vec::new();
        for (key, value) in options {
            if let Some((key, current)) = values.iter_mut().find(|(k, _)| *k == key.as_ref()) {
                match validation::sanitize(key, value) {
                    Ok(value) => *current = value,
                    Err(error) => diagnostics.push(error),
                }
            }
        }
        Self {
            values: Rc::new(RefCell::new(values)),
            changes: Emitter::new(),
            diagnostics,
            dispatch_errors: Rc::new(RefCell::new(Vec::new())),
        }
    }

    pub fn diagnostics(&self) -> &[OptionError] {
        &self.diagnostics
    }

    /// Includes all defaults, in upstream Object.keys order.
    pub fn keys(&self) -> Vec<&'static str> {
        self.values.borrow().iter().map(|(key, _)| *key).collect()
    }

    pub fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        self.values
            .borrow()
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, value)| value.clone())
            .ok_or_else(|| OptionError::UnknownKey(key.into()))
    }

    /// Sanitizes before comparing with JS strict equality. An unchanged value
    /// emits nothing. The value is committed before listeners are invoked.
    pub fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        let key = self.key(key)?;
        let value = validation::sanitize(key, value)?;
        let changed = {
            let mut values = self.values.borrow_mut();
            // key() checked existence, and this service never removes keys.
            let (_, current) = values
                .iter_mut()
                .find(|(k, _)| *k == key)
                .expect("validated option key remains present");
            if *current == value {
                false
            } else {
                *current = value;
                true
            }
        };
        if changed {
            self.dispatch_errors.borrow_mut().push(None);
            let _frame = DispatchFrame(&self.dispatch_errors);
            let result = self.changes.fire(&key);
            if let Some(error) = self
                .dispatch_errors
                .borrow_mut()
                .last_mut()
                .and_then(Option::take)
            {
                return Err(error);
            }
            result.map_err(OptionError::Dispatch)?;
        }
        Ok(())
    }

    /// Internal equivalent of writing rawOptions, bypassing validation/events.
    pub fn set_raw(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        let mut values = self.values.borrow_mut();
        let (_, current) = values
            .iter_mut()
            .find(|(k, _)| *k == key)
            .ok_or_else(|| OptionError::UnknownKey(key.into()))?;
        *current = value;
        Ok(())
    }

    pub fn on_option_change(&self) -> Event<&'static str> {
        self.changes.event()
    }

    pub fn on_specific_option_change(
        &self,
        key: &str,
        listener: impl Fn(OptionValue) + 'static,
    ) -> Result<DisposableHandle, OptionError> {
        self.try_on_specific_option_change(key, move |value| {
            listener(value);
            Ok(())
        })
    }

    /// Fallible listeners model a JS callback throwing. The same error returns
    /// from set after the value commits, and later listeners are not invoked.
    pub fn try_on_option_change(
        &self,
        listener: impl Fn(&'static str) -> Result<(), OptionError> + 'static,
    ) -> DisposableHandle {
        let errors = Rc::downgrade(&self.dispatch_errors);
        self.changes.event().try_subscribe(move |key| {
            if let Err(error) = listener(key) {
                if let Some(errors) = errors.upgrade() {
                    if let Some(frame) = errors.borrow_mut().last_mut() {
                        *frame = Some(error);
                    }
                }
                return Err(EmitError);
            }
            Ok(())
        })
    }

    pub fn try_on_specific_option_change(
        &self,
        key: &str,
        listener: impl Fn(OptionValue) -> Result<(), OptionError> + 'static,
    ) -> Result<DisposableHandle, OptionError> {
        let key = self.key(key)?;
        let values = Rc::downgrade(&self.values);
        Ok(self.try_on_option_change(move |changed| {
            if changed == key {
                if let Some(values) = values.upgrade() {
                    let value = values
                        .borrow()
                        .iter()
                        .find(|(k, _)| *k == key)
                        .map(|(_, value)| value.clone());
                    if let Some(value) = value {
                        listener(value)?;
                    }
                }
            }
            Ok(())
        }))
    }

    pub fn on_multiple_option_change(
        &self,
        keys: &[&str],
        listener: impl Fn() + 'static,
    ) -> Result<DisposableHandle, OptionError> {
        let keys: Vec<_> = keys
            .iter()
            .map(|key| self.key(key))
            .collect::<Result<_, _>>()?;
        Ok(self.changes.event().subscribe(move |changed| {
            if keys.contains(changed) {
                listener();
            }
        }))
    }

    fn key(&self, key: &str) -> Result<&'static str, OptionError> {
        self.values
            .borrow()
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(key, _)| *key)
            .ok_or_else(|| OptionError::UnknownKey(key.into()))
    }
}

impl Default for OptionsService {
    fn default() -> Self {
        Self::new(false, std::iter::empty::<(&str, OptionValue)>())
    }
}

impl Disposable for OptionsService {
    fn dispose(&self) {
        self.changes.dispose();
        for (key, value) in self.values.borrow_mut().iter_mut() {
            if matches!(*key, "linkHandler" | "documentOverride") {
                *value = OptionValue::Null;
            }
        }
    }
}
impl Drop for OptionsService {
    fn drop(&mut self) {
        self.dispose();
    }
}
