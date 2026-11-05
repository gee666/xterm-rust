// Copyright (c) 2016 The xterm.js authors. All rights reserved. MIT license.
use super::{
    storage::{PhysicalRead, Storage},
    CircularList,
};
use std::{cell::RefCell, rc::Rc};

/// A phased numeric lookup that retains the original backing array. The index
/// callback may mutate the list: addition uses the captured start, remainder
/// uses the capacity after the callback, and lookup uses the captured backing.
/// This matches CircularList.get when JavaScript index conversion reenters it.
pub struct CircularListNumberRead<T> {
    storage: Rc<RefCell<Storage<T>>>,
    backing: PhysicalRead<T>,
    start: usize,
}

impl<T: Clone> CircularListNumberRead<T> {
    /// Convert `start + index` without holding a list borrow across user code.
    /// The callback returns the numeric sum, not the index alone. In JavaScript,
    /// string concatenation is therefore evaluated before numeric remainder.
    pub fn get_with<E>(self, add: impl FnOnce(f64) -> Result<f64, E>) -> Result<Option<T>, E> {
        let sum = add(self.start as f64)?;
        let capacity = self.storage.borrow().max_length;
        Ok(self.backing.get_number(sum % capacity as f64))
    }
}

impl<T: Clone> CircularList<T> {
    /// Begin an indexing operation before invoking a possibly reentrant index
    /// conversion. No element copies or storage borrows cross the callback.
    pub fn prepare_number_read(&self) -> CircularListNumberRead<T> {
        let storage = self.storage.borrow();
        let (backing, start) = storage.prepare_read();
        CircularListNumberRead {
            storage: Rc::clone(&self.storage),
            backing,
            start,
        }
    }
}
