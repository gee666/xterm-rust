// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ordered SameValueZero keys for Kitty's JavaScript Maps.

use crate::image::storage::StorageError;
use std::cell::RefCell;
use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::ops::Bound::{Excluded, Unbounded};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct NumberKey(u64);
impl NumberKey {
    fn new(value: f64) -> Self {
        Self(if value == 0.0 {
            0
        } else if value.is_nan() {
            f64::NAN.to_bits()
        } else {
            value.to_bits()
        })
    }
    fn value(self) -> f64 {
        f64::from_bits(self.0)
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OrderExhausted;
impl fmt::Display for OrderExhausted {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("image map insertion ordinal exhausted")
    }
}
impl std::error::Error for OrderExhausted {}

pub(super) struct NumberMap<T> {
    lookup: HashMap<NumberKey, u64>,
    ordered: BTreeMap<u64, (NumberKey, T)>,
    next: u64,
}
impl<T> Default for NumberMap<T> {
    fn default() -> Self {
        Self {
            lookup: HashMap::new(),
            ordered: BTreeMap::new(),
            next: 0,
        }
    }
}
impl<T> NumberMap<T> {
    pub fn len(&self) -> usize {
        self.lookup.len()
    }
    pub fn contains(&self, key: f64) -> bool {
        self.lookup.contains_key(&NumberKey::new(key))
    }
    pub fn get(&self, key: f64) -> Option<&T> {
        let ordinal = self.lookup.get(&NumberKey::new(key))?;
        // Every mutation updates both private indices without a callback between.
        Some(
            &self
                .ordered
                .get(ordinal)
                .expect("number map indices agree")
                .1,
        )
    }
    pub fn insert(&mut self, key: f64, value: T) -> Result<Option<T>, OrderExhausted> {
        let key = NumberKey::new(key);
        if let Some(ordinal) = self.lookup.get(&key) {
            let entry = self
                .ordered
                .get_mut(ordinal)
                .expect("number map indices agree");
            Ok(Some(std::mem::replace(&mut entry.1, value)))
        } else {
            let following = self.next.checked_add(1).ok_or(OrderExhausted)?;
            self.lookup.insert(key, self.next);
            self.ordered.insert(self.next, (key, value));
            self.next = following;
            Ok(None)
        }
    }
    pub fn set(&mut self, key: f64, value: T) -> Result<(), OrderExhausted> {
        self.insert(key, value).map(|_| ())
    }
    pub fn remove(&mut self, key: f64) -> Option<T> {
        let ordinal = self.lookup.remove(&NumberKey::new(key))?;
        Some(
            self.ordered
                .remove(&ordinal)
                .expect("number map indices agree")
                .1,
        )
    }
    pub fn clear(&mut self) {
        drop(self.drain_values());
    }
    /// Detach values without dropping them under an enclosing state borrow.
    pub fn drain_values(&mut self) -> impl Iterator<Item = T> + use<T> {
        self.lookup.clear();
        let ordered = std::mem::take(&mut self.ordered);
        // Keep ordinals monotonic: a live iterator observes appends after clear.
        ordered.into_values().map(|(_, value)| value)
    }
    pub fn after(&self, cursor: Option<u64>) -> Option<(u64, f64, &T)> {
        let next = if let Some(cursor) = cursor {
            self.ordered.range((Excluded(cursor), Unbounded)).next()
        } else {
            self.ordered.first_key_value()
        };
        next.map(|(ordinal, (key, value))| (*ordinal, key.value(), value))
    }
}

type OrderedEntry<T> = (u64, f64, T);
trait MapAccess<T> {
    fn len(&self) -> Result<usize, StorageError>;
    fn get(&self, key: f64) -> Result<Option<T>, StorageError>;
    fn after(&self, cursor: Option<u64>) -> Result<Option<OrderedEntry<T>>, StorageError>;
}
struct StateAccess<S, T> {
    state: Rc<RefCell<S>>,
    select: fn(&S) -> &NumberMap<T>,
}
#[derive(Debug)]
struct BorrowedMap;
impl fmt::Display for BorrowedMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("image map owner is already borrowed")
    }
}
impl std::error::Error for BorrowedMap {}
fn map_borrow(_: impl fmt::Debug) -> StorageError {
    StorageError::host(BorrowedMap)
}
impl<S, T: Clone> MapAccess<T> for StateAccess<S, T> {
    fn len(&self) -> Result<usize, StorageError> {
        let state = self.state.try_borrow().map_err(map_borrow)?;
        Ok((self.select)(&state).len())
    }
    fn get(&self, key: f64) -> Result<Option<T>, StorageError> {
        let state = self.state.try_borrow().map_err(map_borrow)?;
        Ok((self.select)(&state).get(key).cloned())
    }
    fn after(&self, cursor: Option<u64>) -> Result<Option<OrderedEntry<T>>, StorageError> {
        let state = self.state.try_borrow().map_err(map_borrow)?;
        Ok((self.select)(&state)
            .after(cursor)
            .map(|(order, key, value)| (order, key, value.clone())))
    }
}

/// Live read-only number-map identity shared by images, placements and uploads.
#[derive(Clone)]
pub struct NumberMapView<T> {
    access: Rc<dyn MapAccess<T>>,
}
impl<T: Clone + 'static> NumberMapView<T> {
    pub(super) fn new<S: 'static>(state: Rc<RefCell<S>>, select: fn(&S) -> &NumberMap<T>) -> Self {
        Self {
            access: Rc::new(StateAccess { state, select }),
        }
    }
    pub fn same_identity(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.access, &other.access)
    }
    pub fn len(&self) -> Result<usize, StorageError> {
        self.access.len()
    }
    pub fn is_empty(&self) -> Result<bool, StorageError> {
        Ok(self.len()? == 0)
    }
    pub fn get(&self, key: f64) -> Result<Option<T>, StorageError> {
        self.access.get(key)
    }
    pub fn iter(&self) -> NumberMapIterator<T> {
        NumberMapIterator {
            access: Rc::clone(&self.access),
            cursor: None,
            done: false,
        }
    }
    pub fn entries_snapshot(&self) -> Result<Vec<(f64, T)>, StorageError> {
        self.iter().collect()
    }
}

/// Each next observes appends/clear until exhaustion, as a JS Map iterator does.
pub struct NumberMapIterator<T> {
    access: Rc<dyn MapAccess<T>>,
    cursor: Option<u64>,
    done: bool,
}
impl<T> Iterator for NumberMapIterator<T> {
    type Item = Result<(f64, T), StorageError>;
    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }
        match self.access.after(self.cursor) {
            Ok(Some((order, key, value))) => {
                self.cursor = Some(order);
                Some(Ok((key, value)))
            }
            Ok(None) => {
                self.done = true;
                None
            }
            Err(error) => Some(Err(error)),
        }
    }
}
