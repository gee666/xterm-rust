// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{Buffer, BufferError, BufferRef};
use std::cell::{Cell, RefCell};

/// Buffer-specific MutableDisposable contract. The shared lifecycle trait is
/// infallible, so using it here would discard marker/host callback errors. Keep
/// this small typed mirror private rather than changing that public foundation.
/// Raw Set fields and these owner slots deliberately need not agree after a
/// nested reset: source assignment commits the owner after disposing its old value.
pub(super) struct BufferOwner {
    value: RefCell<Option<BufferRef>>,
    disposed: Cell<bool>,
}
impl BufferOwner {
    pub(super) fn new(value: BufferRef) -> Self {
        Self {
            value: RefCell::new(Some(value)),
            disposed: Cell::new(false),
        }
    }
    /// Host attachment must include retained owner values even when reentry
    /// made them differ from the Set's raw fields or hid them after disposal.
    pub(super) fn retained_buffer(&self) -> Option<BufferRef> {
        self.value.borrow().clone()
    }
    pub(super) fn replace(&self, value: BufferRef) -> Result<(), BufferError> {
        if self.disposed.get() {
            return Ok(());
        }
        let old = self.value.borrow().clone();
        // Set reset always supplies a freshly constructed buffer, so the
        // generic MutableDisposable same-value branch is unreachable here.
        if let Some(old) = old {
            Buffer::dispose_shared(&old)?;
        }
        // A callback can install another value. Source overwrites that value
        // with this original argument only after its disposal callback returns.
        let previous = self.value.replace(Some(value));
        drop(previous);
        Ok(())
    }
    pub(super) fn dispose(&self) -> Result<(), BufferError> {
        self.disposed.set(true);
        let old = self.value.borrow().clone();
        if let Some(old) = old {
            Buffer::dispose_shared(&old)?;
        }
        let previous = self.value.take();
        drop(previous);
        Ok(())
    }
}
