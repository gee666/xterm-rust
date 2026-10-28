// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Host adapter ownership only. Buffer state is never copied into this registry.
use super::{Buffer, BufferError, BufferRef, BufferSet, State};
use crate::{buffer::cleanup_owner::CleanupOwnerSlot, event::EmitError};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    fmt,
    rc::{Rc, Weak},
};

pub type CleanupFactory = Rc<dyn Fn(&BufferRef) -> Result<Box<dyn Any>, BufferError>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CleanupFactoryError {
    AlreadyInstalled,
    DetachedDuringCallback,
}
impl fmt::Display for CleanupFactoryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::AlreadyInstalled => "buffer set already has a cleanup factory",
            Self::DetachedDuringCallback => "cleanup factory detached during buffer construction",
        })
    }
}
impl std::error::Error for CleanupFactoryError {}
fn failure(error: CleanupFactoryError) -> BufferError {
    BufferError::Dispatch(EmitError::callback(error))
}

/// Keep this unique registration alive for the host lifetime. Core Buffer
/// disposal cancels its own queue; host detachment separately releases adapters.
pub struct CleanupFactoryHandle {
    registration: Rc<Registration>,
}
impl CleanupFactoryHandle {
    pub fn dispose(&self) {
        self.registration.dispose();
    }
}
impl Drop for CleanupFactoryHandle {
    fn drop(&mut self) {
        self.dispose();
    }
}
impl fmt::Debug for CleanupFactoryHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CleanupFactoryHandle")
            .field("disposed", &self.registration.disposed.get())
            .finish_non_exhaustive()
    }
}

pub(super) struct Registration {
    state: Weak<State>,
    factory: CleanupFactory,
    token: Rc<()>,
    slots: RefCell<Vec<Weak<CleanupOwnerSlot>>>,
    disposed: Cell<bool>,
}
impl Registration {
    fn active(&self) -> Result<(), BufferError> {
        let matches = self.state.upgrade().is_some_and(|state| {
            state
                .factory
                .borrow()
                .as_ref()
                .is_some_and(|current| Rc::ptr_eq(&current.token, &self.token))
        });
        if self.disposed.get() || !matches {
            Err(failure(CleanupFactoryError::DetachedDuringCallback))
        } else {
            Ok(())
        }
    }
    pub(super) fn attach(&self, buffer: &BufferRef) -> Result<(), BufferError> {
        self.active()?;
        let slot = Buffer::cleanup_owner_slot_shared(buffer)?;
        if slot.is_owned_by(&self.token) {
            return Ok(());
        }
        let resource = (self.factory)(buffer)?;
        if let Err(error) = self.active() {
            drop(resource);
            return Err(error);
        }
        {
            let mut slots = self.slots.borrow_mut();
            // Ordinary resets must not accumulate one weak record forever for
            // every obsolete buffer. Live external buffers retain their slot.
            slots.retain(|slot| slot.strong_count() != 0);
            slots.push(Rc::downgrade(&slot));
        }
        slot.install(&self.token, resource);
        self.active()
    }
    fn dispose(&self) {
        if self.disposed.replace(true) {
            return;
        }
        if let Some(state) = self.state.upgrade() {
            let removed = {
                let mut factory = state.factory.borrow_mut();
                if factory
                    .as_ref()
                    .is_some_and(|current| Rc::ptr_eq(&current.token, &self.token))
                {
                    factory.take()
                } else {
                    None
                }
            };
            drop(removed);
        }
        let slots = std::mem::take(&mut *self.slots.borrow_mut());
        for slot in slots {
            if let Some(slot) = slot.upgrade() {
                let resource = slot.take_if_owned(&self.token);
                // Host cancellation/destruction may reset buffers or install a
                // replacement factory. No Set or owner-slot borrow spans it.
                drop(resource);
            }
        }
    }
}

pub(super) fn install(
    set: &BufferSet,
    factory: CleanupFactory,
) -> Result<CleanupFactoryHandle, BufferError> {
    let registration = Rc::new(Registration {
        state: Rc::downgrade(&set.state),
        factory,
        token: Rc::new(()),
        slots: RefCell::new(Vec::new()),
        disposed: Cell::new(false),
    });
    {
        let mut current = set.state.factory.borrow_mut();
        if current.is_some() {
            return Err(failure(CleanupFactoryError::AlreadyInstalled));
        }
        *current = Some(Rc::clone(&registration));
    }
    let handle = CleanupFactoryHandle { registration };
    for buffer in set.retained_buffers() {
        if let Err(error) = handle.registration.attach(&buffer) {
            handle.dispose();
            return Err(error);
        }
    }
    Ok(handle)
}
