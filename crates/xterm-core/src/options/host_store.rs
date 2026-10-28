//! Optional host storage preserves JS object mechanics without moving terminal
//! validation or event ordering into JavaScript. Native services use owned Rust
//! values; browser services use the actual spread-copied raw options object.
use super::{OptionError, OptionValue, OptionsService};
use std::rc::Rc;

/// A binding must implement property access/assignment, including the receiver,
/// descriptors and thrown errors, using its host's object model.
pub trait RawOptionsStore {
    /// Some reflects dynamic host membership; None uses the typed key schema.
    fn validate_key(&self, _key: &str) -> Result<Option<bool>, OptionError> {
        Ok(None)
    }
    /// Host values can have typed adapter views while retaining primitive JS
    /// identity. Some supplies exact host equality; None uses native equality.
    fn strict_equal(
        &self,
        _left: &OptionValue,
        _right: &OptionValue,
    ) -> Result<Option<bool>, OptionError> {
        Ok(None)
    }
    fn get(&self, key: &str) -> Result<OptionValue, OptionError>;
    fn set(&self, key: &str, value: OptionValue) -> Result<(), OptionError>;
}

/// Names inherited from Object.prototype pass the source's `key in defaults`
/// check. __proto__ is handled by the host's setter, not as a data property.
pub(super) fn prototype_key(key: &str) -> Option<&'static str> {
    Some(match key {
        "__proto__" => "__proto__",
        "constructor" => "constructor",
        "toString" => "toString",
        "toLocaleString" => "toLocaleString",
        "valueOf" => "valueOf",
        "hasOwnProperty" => "hasOwnProperty",
        "isPrototypeOf" => "isPrototypeOf",
        "propertyIsEnumerable" => "propertyIsEnumerable",
        "__defineGetter__" => "__defineGetter__",
        "__defineSetter__" => "__defineSetter__",
        "__lookupGetter__" => "__lookupGetter__",
        "__lookupSetter__" => "__lookupSetter__",
        _ => return None,
    })
}

impl OptionsService {
    fn validate_store_key(
        &self,
        key: &str,
        store: &dyn RawOptionsStore,
    ) -> Result<(), OptionError> {
        let installed = self
            .property_layout
            .as_ref()
            .is_some_and(|layout| layout.iter().any(|(name, _)| name == key));
        let allowed = store
            .validate_key(key)?
            .unwrap_or_else(|| self.key(key).is_ok());
        if installed && allowed {
            Ok(())
        } else {
            Err(OptionError::UnknownKey(key.into()))
        }
    }

    pub(super) fn read_value(&self, key: &str) -> Result<OptionValue, OptionError> {
        if let Some(store) = &self.raw_store {
            self.validate_store_key(key, store.as_ref())?;
            return store.get(key);
        }
        self.values
            .borrow()
            .iter()
            .find(|(name, _)| *name == key)
            .map(|(_, value)| value.clone())
            .ok_or_else(|| OptionError::UnknownKey(key.into()))
    }

    pub(super) fn write_value(&self, key: &str, value: OptionValue) -> Result<(), OptionError> {
        if let Some(store) = &self.raw_store {
            self.validate_store_key(key, store.as_ref())?;
            let value = self.sanitize_value(key, value)?;
            let current = store.get(key)?;
            if store
                .strict_equal(&current, &value)?
                .unwrap_or_else(|| current == value)
            {
                return Ok(());
            }
            store.set(key, value)?;
        } else {
            let key = self.key(key)?;
            let value = super::validation::sanitize(key, value)?;
            let current = self.read_value(key)?;
            if current == value {
                return Ok(());
            }
            let previous = {
                let mut values = self.values.borrow_mut();
                let (_, current) = values
                    .iter_mut()
                    .find(|(name, _)| *name == key)
                    .expect("validated option key remains present");
                std::mem::replace(current, value)
            };
            drop(previous);
            drop(current);
        }
        self.changes.fire(&key.to_owned()).map_err(|error| {
            error
                .callback_ref::<OptionError>()
                .cloned()
                .unwrap_or(OptionError::Dispatch(error))
        })
    }

    /// Attach a resolved host object after exactly one constructor sanitization
    /// pass. Layout entries list every core accessor and its enumerability. This
    /// is the binding equivalent of source _setupOptions, not a second validator.
    pub fn with_raw_store(
        mut self,
        store: Rc<dyn RawOptionsStore>,
        layout: Vec<(String, bool)>,
    ) -> Self {
        self.raw_store = Some(store);
        self.property_layout = Some(layout);
        self
    }

    /// JS Object.defineProperty needs names the typed Rust key list cannot
    /// express, including invalid inherited properties whose getter must throw.
    pub fn property_layout(&self) -> Vec<(String, bool)> {
        self.property_layout.clone().unwrap_or_else(|| {
            self.keys()
                .into_iter()
                .map(|key| (key.into(), true))
                .collect()
        })
    }

    /// Source's sanitizer accepts unknown keys without extra validation. The
    /// constructor's host membership check and installed accessors gate access.
    pub fn sanitize_value(
        &self,
        key: &str,
        value: OptionValue,
    ) -> Result<OptionValue, OptionError> {
        let known = self
            .values
            .borrow()
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(key, _)| *key);
        match known {
            Some(key) => super::validation::sanitize(key, value),
            None => Ok(value),
        }
    }

    pub fn try_dispose(&self) -> Result<(), OptionError> {
        use crate::lifecycle::Disposable;
        if self.disposed.replace(true) {
            return Ok(());
        }
        self.changes.dispose();
        // The source callback clears linkHandler before documentOverride.
        self.set_raw("linkHandler", OptionValue::Null)?;
        self.set_raw("documentOverride", OptionValue::Null)
    }

    /// Infallible Disposable/Drop cannot return host exceptions. Explicit
    /// callers can use try_dispose; adapters can retrieve a trait-call failure.
    pub fn take_dispose_error(&self) -> Option<OptionError> {
        self.dispose_error.borrow_mut().take()
    }
}
