// Copyright (c) 2022 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.

//! Sorted, duplicate-preserving storage ported from `common/SortedList.ts`.

use std::cmp::Ordering;

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
    key: F,
}

impl<T, F: Fn(&T) -> f64> SortedList<T, F> {
    pub fn new(key: F) -> Self {
        Self {
            values: Vec::new(),
            inserted: Vec::new(),
            deleted: Vec::new(),
            key,
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
        self.inserted.clear();
        self.deleted.clear();
    }

    pub fn insert(&mut self, value: T) {
        self.flush_deleted();
        self.inserted.push(value);
    }

    /// Delete the first matching value with the same numeric key.
    ///
    /// Like JavaScript strict equality, `NaN` keys never match. The equality
    /// predicate lets callers retain reference identity instead of structural
    /// equality for objects.
    pub fn delete_by(&mut self, value: &T, equal: impl Fn(&T, &T) -> bool) -> bool {
        self.flush_inserted();
        let key = (self.key)(value);
        let mut index = self.search(key);
        while let Some(candidate) = self.values.get(index) {
            if (self.key)(candidate) != key {
                break;
            }
            if equal(candidate, value) {
                self.deleted.push(index);
                return true;
            }
            index += 1;
        }
        false
    }

    pub fn delete(&mut self, value: &T) -> bool
    where
        T: PartialEq,
    {
        self.delete_by(value, PartialEq::eq)
    }

    /// Iterate entries with exactly this key. Borrowing prevents concurrent
    /// list mutation; use [`Self::values`] for an independently owned snapshot.
    pub fn get_key_iterator(&mut self, key: f64) -> impl Iterator<Item = &T> {
        self.flush();
        let start = self.search(key);
        let get_key = &self.key;
        self.values[start..]
            .iter()
            .take_while(move |value| get_key(value) == key)
    }

    pub fn for_each_by_key(&mut self, key: f64, mut callback: impl FnMut(&T)) {
        for value in self.get_key_iterator(key) {
            callback(value);
        }
    }

    /// Snapshot of the list, allowing mutation while visiting its values.
    /// Use reference-counted entries when cloning must preserve object identity.
    pub fn values(&mut self) -> std::vec::IntoIter<T>
    where
        T: Clone,
    {
        self.flush();
        self.values.clone().into_iter()
    }

    pub fn flush(&mut self) {
        self.flush_inserted();
        self.flush_deleted();
    }

    fn flush_inserted(&mut self) {
        if self.inserted.is_empty() {
            return;
        }
        let key = &self.key;
        if self.inserted.iter().any(|value| key(value).is_nan()) {
            // JS treats a NaN comparison as equal. That is not a total order,
            // so Rust's standard sort may panic. Insertion sort tolerates the
            // inconsistent comparator without inventing an order for NaN.
            for index in 1..self.inserted.len() {
                let mut position = index;
                while position > 0
                    && key(&self.inserted[position]) < key(&self.inserted[position - 1])
                {
                    self.inserted.swap(position, position - 1);
                    position -= 1;
                }
            }
        } else {
            self.inserted.sort_by(|a, b| {
                (key(a) - key(b))
                    .partial_cmp(&0.0)
                    .unwrap_or(Ordering::Equal)
            });
        }
        let mut added = std::mem::take(&mut self.inserted).into_iter().peekable();
        let old = std::mem::take(&mut self.values);
        self.values.reserve(old.len() + added.len());
        for value in old {
            while added.peek().is_some_and(|next| key(next) <= key(&value)) {
                // `peek` proved the next entry exists. `for` would consume the
                // first entry belonging after this old value.
                if let Some(next) = added.next() {
                    self.values.push(next);
                }
            }
            self.values.push(value);
        }
        self.values.extend(added);
    }

    fn flush_deleted(&mut self) {
        if self.deleted.is_empty() {
            return;
        }
        self.deleted.sort_unstable();
        self.deleted.dedup();
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
    }

    fn search(&self, key: f64) -> usize {
        let mut min = 0;
        let mut max = self.values.len();
        while min < max {
            let mid = min + (max - min) / 2;
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
