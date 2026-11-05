// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Sparse slots mirror JavaScript array holes without an
// eager allocation at a source-supported capacity of 2^32-1. HashMap reserve
// failures are recoverable; cloning a caller's T retains T's own contract.

use super::CircularListError;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Key {
    Integer(i128),
    Fraction(u64),
    Nan,
}
impl Key {
    fn from_number(value: f64) -> Self {
        if value.is_nan() {
            Self::Nan
        } else if value.fract() == 0.0 {
            // A remainder is bounded by usize capacity, at most 64 bits on
            // supported targets. The integral conversion cannot saturate.
            // Both signs of zero name the same JavaScript array property.
            Self::Integer(value as i128)
        } else {
            Self::Fraction(value.to_bits())
        }
    }
}

pub(super) struct Storage<T> {
    slots: Rc<RefCell<HashMap<Key, T>>>,
    start: usize,
    pub length: usize,
    pub max_length: usize,
}

pub(super) struct PhysicalRead<T> {
    slots: Rc<RefCell<HashMap<Key, T>>>,
}
impl<T: Clone> PhysicalRead<T> {
    pub fn get_number(&self, index: f64) -> Option<T> {
        self.slots.borrow().get(&Key::from_number(index)).cloned()
    }
}

// Supported pointer widths are at most 64 bits; this is a lossless widening.
fn number(index: usize) -> i128 {
    index as i128
}

impl<T: Clone> Storage<T> {
    pub fn new(max_length: usize) -> Self {
        Self {
            slots: Rc::new(RefCell::new(HashMap::new())),
            start: 0,
            length: 0,
            max_length,
        }
    }
    pub fn prepare_read(&self) -> (PhysicalRead<T>, usize) {
        (
            PhysicalRead {
                slots: Rc::clone(&self.slots),
            },
            self.start,
        )
    }
    fn cyclic_index(&self, index: usize) -> Key {
        Key::Integer((number(self.start) + number(index)) % number(self.max_length))
    }
    fn signed_index(&self, index: i64) -> Key {
        Key::Integer((number(self.start) + i128::from(index)) % number(self.max_length))
    }
    fn number_index(&self, index: f64) -> Key {
        // Preserve JS addition rounding before remainder, including fractional
        // properties and the shared "NaN" property reached by either infinity.
        Key::from_number((self.start as f64 + index) % self.max_length as f64)
    }

    /// Push/recycle normalize start; trim/splice/shifts preserve its full value.
    pub fn rotate(&mut self) {
        self.start = (self.start % self.max_length + 1) % self.max_length;
    }
    pub fn advance(&mut self, count: usize) -> Result<(), CircularListError> {
        self.start = self
            .start
            .checked_add(count)
            .ok_or(CircularListError::SizeOverflow)?;
        Ok(())
    }
    pub fn get(&self, index: usize) -> Option<T> {
        self.slots.borrow().get(&self.cyclic_index(index)).cloned()
    }
    pub fn get_signed(&self, index: i64) -> Option<T> {
        self.slots.borrow().get(&self.signed_index(index)).cloned()
    }
    pub fn get_number(&self, index: f64) -> Option<T> {
        self.slots.borrow().get(&self.number_index(index)).cloned()
    }
    pub fn set(
        &mut self,
        index: usize,
        value: &mut Option<T>,
    ) -> Result<Option<T>, CircularListError> {
        self.set_physical(self.cyclic_index(index), value)
    }
    pub fn set_signed(
        &mut self,
        index: i64,
        value: &mut Option<T>,
    ) -> Result<Option<T>, CircularListError> {
        self.set_physical(self.signed_index(index), value)
    }
    pub fn set_number(
        &mut self,
        index: f64,
        value: &mut Option<T>,
    ) -> Result<Option<T>, CircularListError> {
        self.set_physical(self.number_index(index), value)
    }
    pub fn stored_values(&self) -> Result<Vec<T>, CircularListError> {
        let slots = self.slots.borrow();
        let mut values = Vec::new();
        values
            .try_reserve_exact(slots.len())
            .map_err(|_| CircularListError::AllocationFailed)?;
        values.extend(slots.values().cloned());
        Ok(values)
    }
    fn set_physical(
        &mut self,
        index: Key,
        value: &mut Option<T>,
    ) -> Result<Option<T>, CircularListError> {
        // On reserve failure the caller still owns the rejected value, so its
        // destructor also runs only after the storage borrow has ended.
        let mut slots = self.slots.borrow_mut();
        if value.is_some() && !slots.contains_key(&index) {
            reserve_slots(&mut slots, 1)?;
        }
        Ok(match value.take() {
            Some(value) => slots.insert(index, value),
            None => slots.remove(&index),
        })
    }
    pub fn set_length(&mut self, length: usize) -> Vec<T> {
        let removed = if length > self.length {
            let old = number(self.length);
            // Scan materialized slots, not the potentially enormous length
            // range. Detach every cleared value before any destructor reenters.
            let mut slots = self.slots.borrow_mut();
            let keys: Vec<_> = slots.keys().copied().filter(|key| {
                matches!(key, Key::Integer(index) if *index >= old && *index < number(length))
            }).collect();
            keys.into_iter()
                .filter_map(|key| slots.remove(&key))
                .collect()
        } else {
            Vec::new()
        };
        self.length = length;
        removed
    }
    pub fn set_max_length(&mut self, capacity: usize) -> Result<Option<Self>, CircularListError> {
        if capacity == self.max_length {
            return Ok(None);
        }
        let mut slots = HashMap::new();
        let limit = number(capacity.min(self.length));
        // Reserve populated copies before cloning so impossible allocations fail
        // without cloning billions of entries or mutating the original storage.
        let previous = self.slots.borrow();
        let required = previous.keys().try_fold(0usize, |total, &physical| {
            let Some(logical) = self.logical_index(physical) else {
                return Ok(total);
            };
            if logical >= limit {
                return Ok(total);
            }
            let copies = (limit - 1 - logical) / number(self.max_length) + 1;
            let copies = usize::try_from(copies).map_err(|_| CircularListError::SizeOverflow)?;
            total
                .checked_add(copies)
                .ok_or(CircularListError::SizeOverflow)
        })?;
        reserve_slots(&mut slots, required)?;
        for (&physical, value) in previous.iter() {
            let Some(mut logical) = self.logical_index(physical) else {
                continue;
            };
            while logical < limit {
                slots.insert(Key::Integer(logical), value.clone());
                logical += number(self.max_length);
            }
        }
        drop(previous);
        let replacement = Self {
            slots: Rc::new(RefCell::new(slots)),
            start: 0,
            length: self.length,
            max_length: capacity,
        };
        Ok(Some(std::mem::replace(self, replacement)))
    }
    fn logical_index(&self, physical: Key) -> Option<i128> {
        // A new JS array copies only nonnegative cyclic integer slots. Named
        // fractional/NaN and negative properties stay on the discarded array.
        match physical {
            Key::Integer(index) if index >= 0 => {
                Some((index - number(self.start)).rem_euclid(number(self.max_length)))
            }
            _ => None,
        }
    }
}

fn reserve_slots<T>(
    slots: &mut HashMap<Key, T>,
    additional: usize,
) -> Result<(), CircularListError> {
    slots
        .try_reserve(additional)
        .map_err(|_| CircularListError::AllocationFailed)
}
