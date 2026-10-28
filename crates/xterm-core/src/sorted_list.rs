// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
//! Deferred sorted storage. Primitive comparison and getter failure are explicit.
use std::{fmt, rc::Rc};
mod iteration;
mod key;
mod lookup;
mod sort_batch;
mod sort_insert;
pub use iteration::{IterationContext, KeyCursor, KeyItem};
pub use key::{KeyReader, SortedKey, TryKey};
pub use sort_batch::with_unordered_keys as stable_sort_by_js_number;

#[derive(Clone, Debug)]
pub enum SortedListError {
    InvalidArrayLength,
    KeyAccess(String),
    Callback(Rc<dyn std::error::Error>),
    UndefinedYield,
}
pub use SortedListError::InvalidArrayLength;
impl SortedListError {
    pub fn callback(error: impl std::error::Error + 'static) -> Self {
        Self::Callback(Rc::new(error))
    }
}
impl PartialEq for SortedListError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidArrayLength, Self::InvalidArrayLength)
            | (Self::UndefinedYield, Self::UndefinedYield) => true,
            (Self::KeyAccess(a), Self::KeyAccess(b)) => a == b,
            (Self::Callback(a), Self::Callback(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for SortedListError {}
impl fmt::Display for SortedListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArrayLength => f.write_str("Invalid array length"),
            Self::KeyAccess(message) => f.write_str(message),
            Self::Callback(error) => error.fmt(f),
            Self::UndefinedYield => f.write_str(
                "Source iterator yielded undefined; use next_key_item to retain its type",
            ),
        }
    }
}
impl std::error::Error for SortedListError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::Callback(error) = self {
            Some(error.as_ref())
        } else {
            None
        }
    }
}
type Predicate<T> = Box<dyn Fn(&T) -> bool>;
/// Native f64 closures remain a convenience reader. `TryKey` preserves typed
/// Number/String/undefined keys and errors without replacing strict comparison.
pub struct SortedList<T, R> {
    values: Vec<T>,
    inserted: Vec<T>,
    deleted: Vec<usize>,
    deletion_flush_failed: bool,
    iteration_context: IterationContext,
    undefined_key_error: Option<String>,
    undefined_key: Option<Predicate<T>>,
    undefined_value: Option<fn(&T) -> bool>,
    key: R,
}
impl<T, F: Fn(&T) -> f64> SortedList<T, F> {
    pub fn new(key: F) -> Self {
        Self::from_reader(key)
    }
}
impl<T, R: KeyReader<T>> SortedList<T, R> {
    pub fn from_reader(key: R) -> Self {
        Self {
            values: Vec::new(),
            inserted: Vec::new(),
            deleted: Vec::new(),
            deletion_flush_failed: false,
            iteration_context: IterationContext::default(),
            undefined_key_error: None,
            undefined_key: None,
            undefined_value: None,
            key,
        }
    }
    pub fn with_undefined_key_error(mut self, message: impl Into<String>) -> Self {
        self.undefined_key_error = Some(message.into());
        self
    }
    pub fn with_undefined_key(mut self, predicate: impl Fn(&T) -> bool + 'static) -> Self {
        self.undefined_key = Some(Box::new(predicate));
        self
    }
    /// Pure representation metadata, not a user key getter. JS Array.sort puts
    /// actual undefined elements last without invoking the comparison callback.
    pub fn with_undefined_values(mut self, predicate: fn(&T) -> bool) -> Self {
        self.undefined_value = Some(predicate);
        self
    }
    fn is_undefined_value(&self, value: &T) -> bool {
        self.undefined_value
            .is_some_and(|predicate| predicate(value))
    }
    fn read_key(&self, value: Option<&T>) -> Result<SortedKey, SortedListError> {
        let undefined = value.is_none_or(|value| self.is_undefined_value(value));
        let key = if undefined {
            if let Some(message) = &self.undefined_key_error {
                return Err(SortedListError::KeyAccess(message.clone()));
            }
            self.key.read_undefined(value)?
        } else {
            self.key.read_key(value)?
        };
        if value.is_some_and(|value| {
            self.undefined_key
                .as_ref()
                .is_some_and(|predicate| predicate(value))
        }) {
            Ok(SortedKey::Undefined)
        } else {
            Ok(key)
        }
    }
    pub fn clear(&mut self) {
        self.values.clear();
        self.inserted.clear();
        self.deleted.clear();
        self.deletion_flush_failed = false;
    }
    pub fn insert(&mut self, value: T) -> Result<(), SortedListError> {
        self.flush_deleted()?;
        self.inserted.push(value);
        Ok(())
    }
    pub fn delete_by(
        &mut self,
        value: &T,
        equal: impl Fn(&T, &T) -> bool,
    ) -> Result<bool, SortedListError> {
        self.flush_inserted()?;
        if self.values.is_empty() {
            return Ok(false);
        }
        let key = self.read_key(Some(value))?;
        if key.is_undefined() {
            return Ok(false);
        }
        let index = self.search(&key)?;
        self.iteration_context.0.set(index);
        if !self.read_key(self.values.get(index))?.strict_eq(&key) {
            return Ok(false);
        }
        loop {
            let index = self.iteration_context.0.get();
            let matches = match self.values.get(index) {
                Some(candidate) => equal(candidate, value),
                None => self.is_undefined_value(value),
            };
            if matches {
                self.deleted.push(index);
                return Ok(true);
            }
            let next = self
                .iteration_context
                .0
                .get()
                .checked_add(1)
                .ok_or(InvalidArrayLength)?;
            self.iteration_context.0.set(next);
            if next >= self.values.len() || !self.read_key(self.values.get(next))?.strict_eq(&key) {
                return Ok(false);
            }
        }
    }
    pub fn delete(&mut self, value: &T) -> Result<bool, SortedListError>
    where
        T: PartialEq,
    {
        self.delete_by(value, PartialEq::eq)
    }
    /// Owned snapshot preserves source values() traversal when the list mutates.
    pub fn values(&mut self) -> Result<std::vec::IntoIter<T>, SortedListError>
    where
        T: Clone,
    {
        self.flush()?;
        Ok(self.values.clone().into_iter())
    }
    pub fn flush(&mut self) -> Result<(), SortedListError> {
        self.flush_inserted()?;
        self.flush_deleted()
    }
    fn flush_deleted(&mut self) -> Result<(), SortedListError> {
        if self.deleted.is_empty() || self.deletion_flush_failed {
            return Ok(());
        }
        self.deleted.sort_unstable();
        if self.deleted.len() > self.values.len() {
            self.deletion_flush_failed = true;
            return Err(InvalidArrayLength);
        }
        // Repeated indices deliberately leave the source deletion cursor behind.
        let mut deleted = self.deleted.iter().copied().peekable();
        let mut index = 0;
        self.values.retain(|_| {
            let remove = deleted.peek() == Some(&index);
            if remove {
                deleted.next();
            }
            index += 1;
            !remove
        });
        self.deleted.clear();
        Ok(())
    }
}
