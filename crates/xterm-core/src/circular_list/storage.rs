// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under MIT. Sparse slots mirror JavaScript array holes without an
// eager allocation at a source-supported capacity of 2^32-1. HashMap reserve
// failures are recoverable; cloning a caller's T retains T's own contract.

use super::CircularListError;
use std::collections::HashMap;

pub(super) struct Storage<T> {
    slots: HashMap<i128, T>,
    start: usize,
    pub length: usize,
    pub max_length: usize,
}

// Rust's supported targets have at most 64-bit pointers. Widening usize to i128
// preserves every index and allows signed remainder without integer overflow.
fn number(index: usize) -> i128 {
    index as i128
}

impl<T: Clone> Storage<T> {
    pub fn new(max_length: usize) -> Self {
        Self {
            slots: HashMap::new(),
            start: 0,
            length: 0,
            max_length,
        }
    }

    fn cyclic_index(&self, index: usize) -> i128 {
        (number(self.start) + number(index)) % number(self.max_length)
    }

    fn signed_index(&self, index: i64) -> i128 {
        (number(self.start) + i128::from(index)) % number(self.max_length)
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
        self.slots.get(&self.cyclic_index(index)).cloned()
    }

    pub fn get_signed(&self, index: i64) -> Option<T> {
        self.slots.get(&self.signed_index(index)).cloned()
    }

    pub fn get_number(&self, index: f64) -> Option<T> {
        // Preserve JS addition rounding before remainder. An integer modulo of
        // the original index would give different slots beyond 2^53.
        let physical = (self.start as f64 + index) % self.max_length as f64;
        if !physical.is_finite() || physical.fract() != 0.0 {
            return None;
        }
        // The finite remainder is bounded by usize capacity, at most 64 bits,
        // so widening that integral value to i128 cannot saturate or truncate.
        self.slots.get(&(physical as i128)).cloned()
    }

    pub fn set(&mut self, index: usize, value: Option<T>) -> Result<(), CircularListError> {
        self.set_physical(self.cyclic_index(index), value)
    }

    pub fn set_signed(&mut self, index: i64, value: Option<T>) -> Result<(), CircularListError> {
        self.set_physical(self.signed_index(index), value)
    }

    fn set_physical(&mut self, index: i128, value: Option<T>) -> Result<(), CircularListError> {
        match value {
            Some(value) => {
                if !self.slots.contains_key(&index) {
                    reserve_slots(&mut self.slots, 1)?;
                }
                self.slots.insert(index, value);
            }
            None => {
                self.slots.remove(&index);
            }
        }
        Ok(())
    }

    pub fn set_length(&mut self, length: usize) {
        if length > self.length {
            let old = number(self.length);
            self.slots
                .retain(|&index, _| index < old || index >= number(length));
        }
        self.length = length;
    }

    pub fn set_max_length(&mut self, capacity: usize) -> Result<(), CircularListError> {
        if capacity == self.max_length {
            return Ok(());
        }
        let mut slots = HashMap::new();
        let limit = number(capacity.min(self.length));
        // Reserve the actual populated copies before cloning. A size failure
        // must not copy billions of entries before discovering the bad size.
        let required = self.slots.keys().try_fold(0usize, |total, &physical| {
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
        for (&physical, value) in &self.slots {
            let Some(mut logical) = self.logical_index(physical) else {
                continue;
            };
            while logical < limit {
                slots.insert(logical, value.clone());
                logical += number(self.max_length);
            }
        }
        self.slots = slots;
        self.max_length = capacity;
        self.start = 0;
        Ok(())
    }

    fn logical_index(&self, physical: i128) -> Option<i128> {
        // Negative array properties are not copied by upstream's indexed loop.
        (physical >= 0).then(|| (physical - number(self.start)).rem_euclid(number(self.max_length)))
    }
}

fn reserve_slots<T>(
    slots: &mut HashMap<i128, T>,
    additional: usize,
) -> Result<(), CircularListError> {
    slots
        .try_reserve(additional)
        .map_err(|_| CircularListError::AllocationFailed)
}
