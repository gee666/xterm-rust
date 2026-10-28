// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ordered SameValueZero keys for Kitty's JavaScript Maps.

use std::collections::{BTreeMap, HashMap};
use std::fmt;
use std::ops::Bound::{Excluded, Unbounded};

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
    pub fn set(&mut self, key: f64, value: T) -> Result<(), OrderExhausted> {
        let key = NumberKey::new(key);
        if let Some(ordinal) = self.lookup.get(&key) {
            self.ordered
                .get_mut(ordinal)
                .expect("number map indices agree")
                .1 = value;
        } else {
            let following = self.next.checked_add(1).ok_or(OrderExhausted)?;
            self.lookup.insert(key, self.next);
            self.ordered.insert(self.next, (key, value));
            self.next = following;
        }
        Ok(())
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
        self.lookup.clear();
        self.ordered.clear();
        // Keep ordinals monotonic: a live iterator can observe appends after clear.
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
