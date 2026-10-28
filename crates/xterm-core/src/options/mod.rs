// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Adapted from common/services/OptionsService.ts.

//! Options at the terminal's JS-compatible boundary. Values deliberately retain
//! upstream permissiveness instead of silently rejecting values that JS accepts.
//! Engine consumers should validate the type they require before using a value.
//! Platform detection is supplied by adapters through `is_mac`.
mod defaults;
mod host_store;
mod object;
mod properties;
mod validation;
mod value;

use crate::event::{EmitError, Emitter, Event};
use crate::lifecycle::{Disposable, DisposableHandle};
pub use defaults::default_options;
pub use host_store::RawOptionsStore;
pub use object::{CoercionError, CoercionHint, NumericSemantics, OptionObject};
pub use properties::{NativeProperties, PropertyAccess};
use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;
pub use value::OptionValue;

/// Native configuration builder for a mutable windowsPty data bag. Absence
/// differs from zero; consumers preserve source truthiness checks.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WindowsPty {
    pub backend: Option<String>,
    pub build_number: Option<f64>,
}

impl From<WindowsPty> for OptionValue {
    fn from(value: WindowsPty) -> Self {
        let mut properties = Vec::new();
        if let Some(backend) = value.backend {
            properties.push(("backend", Self::from(backend.as_str())));
        }
        if let Some(build) = value.build_number {
            properties.push(("buildNumber", Self::Number(build)));
        }
        Self::property_map(properties)
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
impl std::error::Error for OptionError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Coercion(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            _ => None,
        }
    }
}

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
impl From<EmitError> for OptionError {
    fn from(error: EmitError) -> Self {
        Self::Dispatch(error)
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
    changes: Emitter<String>,
    any_changes: Event<String>,
    known_changes: Event<&'static str>,
    diagnostics: Vec<OptionError>,
    raw_store: Option<Rc<dyn RawOptionsStore>>,
    property_layout: Option<Vec<(String, bool)>>,
    dispose_error: RefCell<Option<OptionError>>,
    disposed: Cell<bool>,
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
            } else if let Some(key) =
                host_store::prototype_key(key.as_ref()).filter(|key| *key != "__proto__")
            {
                values.push((key, value));
            }
        }
        let changes = Emitter::new();
        let any_changes = changes.event();
        let keys: Vec<_> = values.iter().map(|(key, _)| *key).collect();
        let known_changes = any_changes.filter_map(move |key: &String| {
            keys.iter()
                .copied()
                .find(|known| *known == key)
                .or_else(|| host_store::prototype_key(key))
        });
        Self {
            values: Rc::new(RefCell::new(values)),
            changes,
            any_changes,
            known_changes,
            diagnostics,
            raw_store: None,
            property_layout: None,
            dispose_error: RefCell::new(None),
            disposed: Cell::new(false),
        }
    }

    pub fn diagnostics(&self) -> &[OptionError] {
        &self.diagnostics
    }

    /// Includes all defaults, in upstream Object.keys order.
    pub fn keys(&self) -> Vec<&'static str> {
        if let Some(layout) = &self.property_layout {
            return layout
                .iter()
                .filter(|(_, enumerable)| *enumerable)
                .filter_map(|(key, _)| self.key(key).ok())
                .collect();
        }
        self.values.borrow().iter().map(|(key, _)| *key).collect()
    }

    pub fn get(&self, key: &str) -> Result<OptionValue, OptionError> {
        self.read_value(key)
    }

    /// Sanitizes before comparing with JS strict equality. An unchanged value
    /// emits nothing. The value is committed before listeners are invoked.
    pub fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        self.write_value(key, value)
    }

    /// Internal equivalent of writing rawOptions, bypassing validation/events.
    pub fn set_raw(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        if let Some(store) = &self.raw_store {
            return store.set(key, value);
        }
        let previous = {
            let mut values = self.values.borrow_mut();
            let (_, current) = values
                .iter_mut()
                .find(|(k, _)| *k == key)
                .ok_or_else(|| OptionError::UnknownKey(key.into()))?;
            std::mem::replace(current, value)
        };
        // Adapter payload destructors may reenter options/logging.
        drop(previous);
        Ok(())
    }

    /// Typed convenience event for declared options and standard prototype names.
    /// Use on_any_option_change for arbitrary host-prototype property names.
    pub fn on_option_change(&self) -> Event<&'static str> {
        self.known_changes.clone()
    }

    /// Complete source onOptionChange, with owned dynamic JS property names.
    pub fn on_any_option_change(&self) -> Event<String> {
        self.any_changes.clone()
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
        self.on_option_change()
            .try_subscribe(move |key| listener(key).map_err(EmitError::callback))
    }

    pub fn try_on_specific_option_change(
        &self,
        key: &str,
        listener: impl Fn(OptionValue) -> Result<(), OptionError> + 'static,
    ) -> Result<DisposableHandle, OptionError> {
        let key = self.key(key)?;
        let values = Rc::downgrade(&self.values);
        let store = self.raw_store.as_ref().map(Rc::downgrade);
        Ok(self.try_on_option_change(move |changed| {
            if changed == key {
                if let Some(store) = store.as_ref().and_then(std::rc::Weak::upgrade) {
                    return listener(store.get(key)?);
                }
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
        Ok(self.on_option_change().subscribe(move |changed| {
            if keys.contains(changed) {
                listener();
            }
        }))
    }

    fn key(&self, key: &str) -> Result<&'static str, OptionError> {
        if self
            .property_layout
            .as_ref()
            .is_some_and(|layout| layout.iter().any(|(name, _)| name == key))
        {
            if let Some(key) = host_store::prototype_key(key) {
                return Ok(key);
            }
        }
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
        if !self.disposed.get() {
            *self.dispose_error.borrow_mut() = self.try_dispose().err();
        }
    }
}
impl Drop for OptionsService {
    fn drop(&mut self) {
        self.dispose();
    }
}
