// Copyright (c) 2016 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Adapted from common/CircularList.ts.

//! Circular backing storage, including upstream's physical-slot length setter.
//! Values are cloned when slots are copied. Use `Rc` elements for JavaScript-like
//! shared object identity. Capacity must be positive; invalid size/range arguments
//! are rejected before mutation instead of producing negative JavaScript lengths.
//! No storage borrow crosses an event callback, permitting reentrant mutation.

use crate::event::{EmitError, Emitter};
use crate::lifecycle::Disposable;
use std::cell::RefCell;
use std::fmt;

#[path = "circular_list/storage.rs"]
mod storage;
use storage::Storage;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ListChange {
    pub index: usize,
    pub amount: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CircularListError {
    ZeroCapacity,
    InvalidRange,
    ShiftBeforeStart,
    NotFull,
    SizeOverflow,
    AllocationFailed,
    Dispatch(EmitError),
}

impl fmt::Display for CircularListError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::ZeroCapacity => "circular list capacity must be positive",
            Self::InvalidRange => "start argument out of range",
            Self::ShiftBeforeStart => "Cannot shift elements in list beyond index 0",
            Self::NotFull => "Can only recycle when the buffer is full",
            Self::SizeOverflow => "circular list size overflow",
            Self::AllocationFailed => "circular list slot allocation failed",
            Self::Dispatch(error) => return error.fmt(f),
        })
    }
}

impl std::error::Error for CircularListError {}
impl From<EmitError> for CircularListError {
    fn from(value: EmitError) -> Self {
        Self::Dispatch(value)
    }
}

pub struct CircularList<T> {
    storage: RefCell<Storage<T>>,
    pub on_delete_emitter: Emitter<ListChange>,
    pub on_insert_emitter: Emitter<ListChange>,
    pub on_trim_emitter: Emitter<usize>,
}

impl<T: Clone> CircularList<T> {
    pub fn new(max_length: usize) -> Result<Self, CircularListError> {
        if max_length == 0 {
            return Err(CircularListError::ZeroCapacity);
        }
        Ok(Self {
            storage: RefCell::new(Storage::new(max_length)),
            on_delete_emitter: Emitter::new(),
            on_insert_emitter: Emitter::new(),
            on_trim_emitter: Emitter::new(),
        })
    }

    pub fn max_length(&self) -> usize {
        self.storage.borrow().max_length
    }
    pub fn len(&self) -> usize {
        self.storage.borrow().length
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn is_full(&self) -> bool {
        self.len() == self.max_length()
    }

    /// Capacity changes copy only live entries and do not change logical length.
    /// Upstream callers shrink `length` separately when reducing capacity.
    pub fn set_max_length(&self, capacity: usize) -> Result<(), CircularListError> {
        if capacity == 0 {
            return Err(CircularListError::ZeroCapacity);
        }
        self.storage.borrow_mut().set_max_length(capacity)
    }

    /// New slots are cleared by physical index, not cyclic index, as upstream.
    pub fn set_length(&self, length: usize) {
        self.storage.borrow_mut().set_length(length);
    }

    /// Indexing deliberately wraps regardless of logical length.
    pub fn get(&self, index: usize) -> Option<T> {
        self.storage.borrow().get(index)
    }

    pub fn set(&self, index: usize, value: Option<T>) -> Result<(), CircularListError> {
        self.storage.borrow_mut().set(index, value)
    }

    /// Preserve JavaScript's signed remainder and negative array-property slots.
    pub fn get_signed(&self, index: i64) -> Option<T> {
        self.storage.borrow().get_signed(index)
    }

    pub fn set_signed(&self, index: i64, value: Option<T>) -> Result<(), CircularListError> {
        self.storage.borrow_mut().set_signed(index, value)
    }

    pub fn push(&self, value: T) -> Result<(), CircularListError> {
        let length = self
            .len()
            .checked_add(1)
            .ok_or(CircularListError::SizeOverflow)?;
        self.set(self.len(), Some(value))?;
        if self.is_full() {
            self.storage.borrow_mut().rotate();
            self.on_trim_emitter.fire(&1)?;
        } else {
            self.storage.borrow_mut().length = length;
        }
        Ok(())
    }

    pub fn recycle(&self) -> Result<Option<T>, CircularListError> {
        if !self.is_full() {
            return Err(CircularListError::NotFull);
        }
        self.storage.borrow_mut().rotate();
        self.on_trim_emitter.fire(&1)?;
        Ok(match self.len().checked_sub(1) {
            Some(index) => self.get(index),
            None => self.get_signed(-1),
        })
    }

    /// The slot is not cleared, matching upstream's observable wrapped indexing.
    /// Popping an empty list is rejected instead of creating a negative length.
    pub fn pop(&self) -> Result<Option<T>, CircularListError> {
        let length = self
            .len()
            .checked_sub(1)
            .ok_or(CircularListError::InvalidRange)?;
        self.storage.borrow_mut().length = length;
        Ok(self.get(length))
    }

    pub fn splice(
        &self,
        start: usize,
        delete_count: usize,
        items: &[T],
    ) -> Result<(), CircularListError> {
        if start > self.len() || delete_count > self.len() - start {
            return Err(CircularListError::InvalidRange);
        }
        self.len()
            .checked_add(items.len())
            .ok_or(CircularListError::SizeOverflow)?;
        if delete_count != 0 {
            for index in start..self.len() - delete_count {
                self.set(index, self.get(index + delete_count))?;
            }
            self.storage.borrow_mut().length -= delete_count;
            self.on_delete_emitter.fire(&ListChange {
                index: start,
                amount: delete_count,
            })?;
        }
        for index in (start..self.len()).rev() {
            self.set(index + items.len(), self.get(index))?;
        }
        for (index, value) in items.iter().enumerate() {
            self.set(start + index, Some(value.clone()))?;
        }
        if !items.is_empty() {
            self.on_insert_emitter.fire(&ListChange {
                index: start,
                amount: items.len(),
            })?;
        }
        let expanded = self
            .len()
            .checked_add(items.len())
            .ok_or(CircularListError::SizeOverflow)?;
        if expanded > self.max_length() {
            let trim = expanded - self.max_length();
            {
                let mut state = self.storage.borrow_mut();
                state.advance(trim)?;
                state.length = state.max_length;
            }
            self.on_trim_emitter.fire(&trim)?;
        } else {
            self.storage.borrow_mut().length = expanded;
        }
        Ok(())
    }

    pub fn trim_start(&self, count: usize) -> Result<(), CircularListError> {
        let count = count.min(self.len());
        {
            let mut state = self.storage.borrow_mut();
            state.advance(count)?;
            state.length -= count;
        }
        self.on_trim_emitter.fire(&count)?;
        Ok(())
    }

    pub fn shift_elements(
        &self,
        start: isize,
        count: usize,
        offset: isize,
    ) -> Result<(), CircularListError> {
        if count == 0 {
            return Ok(());
        }
        let start = usize::try_from(start).map_err(|_| CircularListError::InvalidRange)?;
        if start >= self.len() {
            return Err(CircularListError::InvalidRange);
        }
        let destination = start.checked_add_signed(offset).ok_or(if offset < 0 {
            CircularListError::ShiftBeforeStart
        } else {
            CircularListError::SizeOverflow
        })?;
        let end = destination
            .checked_add(count)
            .ok_or(CircularListError::SizeOverflow)?;
        start
            .checked_add(count)
            .ok_or(CircularListError::SizeOverflow)?;
        if offset > 0 {
            for index in (0..count).rev() {
                self.set(destination + index, self.get(start + index))?;
            }
            if end > self.len() {
                self.storage.borrow_mut().length = end;
                while self.len() > self.max_length() {
                    {
                        let mut state = self.storage.borrow_mut();
                        state.length -= 1;
                        state.advance(1)?;
                    }
                    self.on_trim_emitter.fire(&1)?;
                }
            }
        } else {
            for index in 0..count {
                self.set(destination + index, self.get(start + index))?;
            }
        }
        Ok(())
    }
}

impl<T> Disposable for CircularList<T> {
    fn dispose(&self) {
        self.on_delete_emitter.dispose();
        self.on_insert_emitter.dispose();
        self.on_trim_emitter.dispose();
    }
}
