// Copyright (c) 2022 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.

//! Sorted, duplicate-preserving storage ported from `common/SortedList.ts`.

use std::cmp::Ordering;
use std::fmt;

mod iteration;
mod sort_batch;
pub use iteration::{IterationContext, KeyCursor};

/// Recoverable JavaScript failures exposed by deferred batch operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SortedListError {
    InvalidArrayLength,
    KeyAccess(String),
}

pub use SortedListError::InvalidArrayLength;

impl fmt::Display for SortedListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArrayLength => f.write_str("Invalid array length"),
            Self::KeyAccess(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for SortedListError {}

type UndefinedKeyPredicate<T> = Box<dyn Fn(&T) -> bool>;

/// A list with numeric keys and deferred insertion/deletion batches.
///
/// Reads flush pending changes. New batches precede old entries with equal keys;
/// insertion order within a batch is stable. Object identity is represented by
/// the predicate passed to [`Self::delete_by`], for example `Rc::ptr_eq`.
/// Native Rust has no idle scheduler: batches flush on reads or [`Self::flush`].
pub struct SortedList<T, F> {
    values: Vec<T>,
    inserted: Vec<T>,
    deleted: Vec<usize>,
    deletion_flush_failed: bool,
    iteration_context: IterationContext,
    undefined_key_error: Option<String>,
    undefined_key: Option<UndefinedKeyPredicate<T>>,
    key: F,
}

impl<T, F: Fn(&T) -> f64> SortedList<T, F> {
    pub fn new(key: F) -> Self {
        Self {
            values: Vec::new(),
            inserted: Vec::new(),
            deleted: Vec::new(),
            deletion_flush_failed: false,
            iteration_context: IterationContext::default(),
            undefined_key_error: None,
            undefined_key: None,
            key,
        }
    }

    /// Configure the error thrown by a JS getter such as `entry => entry.key`
    /// when an exhausted pending batch passes `undefined` to that getter.
    /// Scalar identity getters instead return undefined, the default behavior.
    pub fn with_undefined_key_error(mut self, message: impl Into<String>) -> Self {
        self.undefined_key_error = Some(message.into());
        self
    }

    /// Distinguish JS undefined from NaN for getters represented by Option.
    /// Both compare unordered, but deletion of undefined returns before search
    /// and therefore must not change the shared iteration work index.
    pub fn with_undefined_key(mut self, predicate: impl Fn(&T) -> bool + 'static) -> Self {
        self.undefined_key = Some(Box::new(predicate));
        self
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

    /// Delete the first matching value with the same numeric key.
    ///
    /// Like JavaScript strict equality, `NaN` keys never match. The equality
    /// predicate lets callers retain reference identity instead of structural
    /// equality for objects.
    pub fn delete_by(
        &mut self,
        value: &T,
        equal: impl Fn(&T, &T) -> bool,
    ) -> Result<bool, SortedListError> {
        self.flush_inserted()?;
        if self.values.is_empty() {
            return Ok(false);
        }
        let key = (self.key)(value);
        if self
            .undefined_key
            .as_ref()
            .is_some_and(|predicate| predicate(value))
        {
            return Ok(false);
        }
        let mut index = self.search(key);
        self.iteration_context.0.set(index);
        if index >= self.values.len() {
            if let Some(message) = &self.undefined_key_error {
                return Err(SortedListError::KeyAccess(message.clone()));
            }
            return Ok(false);
        }
        while let Some(candidate) = self.values.get(index) {
            if (self.key)(candidate) != key {
                break;
            }
            if equal(candidate, value) {
                self.deleted.push(index);
                return Ok(true);
            }
            index += 1;
            self.iteration_context.0.set(index);
        }
        Ok(false)
    }

    pub fn delete(&mut self, value: &T) -> Result<bool, SortedListError>
    where
        T: PartialEq,
    {
        self.delete_by(value, PartialEq::eq)
    }

    /// Iterate entries with exactly this key. Borrowing prevents concurrent
    /// list mutation; use [`Self::values`] for an independently owned snapshot.
    pub fn get_key_iterator(
        &mut self,
        key: f64,
    ) -> Result<impl Iterator<Item = &T>, SortedListError> {
        self.flush()?;
        let start = self.search(key);
        let get_key = &self.key;
        Ok(self.values[start..]
            .iter()
            .take_while(move |value| get_key(value) == key))
    }

    pub fn for_each_by_key(
        &mut self,
        key: f64,
        mut callback: impl FnMut(&T),
    ) -> Result<(), SortedListError> {
        for value in self.get_key_iterator(key)? {
            callback(value);
        }
        Ok(())
    }

    /// Snapshot of the list, allowing mutation while visiting its values.
    /// Use reference-counted entries when cloning must preserve object identity.
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

    fn flush_inserted(&mut self) -> Result<(), SortedListError> {
        if self.inserted.is_empty() {
            return Ok(());
        }
        let key = &self.key;
        if self.inserted.iter().any(|value| key(value).is_nan()) {
            // JS treats NaN comparisons as equal, an inconsistent comparator
            // that Rust's standard sort may reject. Use a stable binary sort
            // with the same initial-run rule as the observed V8 oracle.
            sort_batch::with_unordered_keys(&mut self.inserted, key);
        } else {
            self.inserted.sort_by(|a, b| {
                (key(a) - key(b))
                    .partial_cmp(&0.0)
                    .unwrap_or(Ordering::Equal)
            });
        }
        // Plan before moving values so a failing JS undefined getter retains
        // the old array and the sorted pending batch, exactly as upstream.
        let total = self
            .values
            .len()
            .checked_add(self.inserted.len())
            .ok_or(InvalidArrayLength)?;
        let mut from_added = Vec::with_capacity(total);
        let (mut old, mut added) = (0, 0);
        while from_added.len() < total {
            let choose_added = if old >= self.values.len() {
                true
            } else if let Some(next) = self.inserted.get(added) {
                key(next) <= key(&self.values[old])
            } else {
                if let Some(message) = &self.undefined_key_error {
                    return Err(SortedListError::KeyAccess(message.clone()));
                }
                false
            };
            from_added.push(choose_added);
            if choose_added {
                added += 1;
            } else {
                old += 1;
            }
        }
        let mut old = std::mem::take(&mut self.values).into_iter();
        let mut added = std::mem::take(&mut self.inserted).into_iter();
        self.values.reserve(from_added.len());
        for choose_added in from_added {
            // Each plan entry increments one bounded source index, and there
            // are exactly old.len()+added.len() entries, so neither is exhausted.
            let value = if choose_added {
                added.next()
            } else {
                old.next()
            }
            .expect("merge plan selects one existing source entry");
            self.values.push(value);
        }
        Ok(())
    }

    fn flush_deleted(&mut self) -> Result<(), SortedListError> {
        if self.deleted.is_empty() || self.deletion_flush_failed {
            return Ok(());
        }
        self.deleted.sort_unstable();
        if self.deleted.len() > self.values.len() {
            // Upstream sets _isFlushingDeleted before allocating. On error it
            // stays set, suppressing later deletion flushes until clear().
            self.deletion_flush_failed = true;
            return Err(InvalidArrayLength);
        }
        // Do not deduplicate. A repeated index leaves the upstream deletion
        // cursor behind and causes all subsequent requested removals to stay.

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

    fn search(&self, key: f64) -> usize {
        let mut min = 0;
        let mut max = self.values.len();
        while min < max {
            // The source uses inclusive max and chooses the left midpoint.
            // Keep that choice for mutable or non-total keys too.
            let mid = min + (max - min - 1) / 2;
            let mid_key = (self.key)(&self.values[mid]);
            if mid_key > key {
                max = mid;
            } else if mid_key < key {
                min = mid + 1;
            } else {
                let mut first = mid;
                while first > 0 && (self.key)(&self.values[first - 1]) == key {
                    first -= 1;
                }
                return first;
            }
        }
        min
    }
}
