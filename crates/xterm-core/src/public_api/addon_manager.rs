// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Public addon ownership. A managed handle replaces JavaScript's instance.dispose
//! method replacement. Explicit disposal returns callback errors; Drop releases
//! Rust-owned resources but does not invoke fallible user callbacks.

use std::{
    cell::{Cell, RefCell},
    fmt,
    rc::{Rc, Weak},
};

/// An addon operates on the actual public terminal supplied at activation.
/// Callbacks use shared receivers so managed disposal can be reentrant.
pub trait TerminalAddon<T> {
    type Error;
    fn activate(&self, terminal: &T) -> Result<(), Self::Error>;
    fn dispose(&self) -> Result<(), Self::Error>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum AddonError<E> {
    Callback(E),
    NotLoaded,
    TraversalInvalidated,
}
impl<E: fmt::Display> fmt::Display for AddonError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Callback(error) => error.fmt(f),
            Self::NotLoaded => f.write_str("Could not dispose an addon that has not been loaded"),
            Self::TraversalInvalidated => {
                f.write_str("addon array changed during reverse disposal")
            }
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for AddonError<E> {}

type EntryList<T, E> = RefCell<Vec<Rc<Entry<T, E>>>>;
type Entries<T, E> = Rc<EntryList<T, E>>;
struct Entry<T, E> {
    instance: Rc<dyn TerminalAddon<T, Error = E>>,
    disposed: Cell<bool>,
}

/// Registration identity. Dispose through this handle rather than calling the
/// raw addon callback, which cannot remove itself from the manager in Rust.
pub struct AddonHandle<T, E> {
    entries: Entries<T, E>,
    entry: Rc<Entry<T, E>>,
}
impl<T, E> Clone for AddonHandle<T, E> {
    fn clone(&self) -> Self {
        Self {
            entries: Rc::clone(&self.entries),
            entry: Rc::clone(&self.entry),
        }
    }
}
impl<T, E> AddonHandle<T, E> {
    pub fn is_disposed(&self) -> bool {
        self.entry.disposed.get()
    }
    pub fn dispose(&self) -> Result<(), AddonError<E>> {
        if self.is_disposed() {
            return Ok(());
        }
        let index = self
            .entries
            .borrow()
            .iter()
            .position(|entry| Rc::ptr_eq(entry, &self.entry))
            .ok_or(AddonError::NotLoaded)?;
        self.entry.disposed.set(true);
        self.entry
            .instance
            .dispose()
            .map_err(AddonError::Callback)?;
        // Upstream splice uses the index captured before the callback, even when
        // that callback changes the array. An out-of-range splice removes nothing.
        let mut entries = self.entries.borrow_mut();
        if index < entries.len() {
            entries.remove(index);
        }
        Ok(())
    }
}

pub struct AddonManager<T, E> {
    entries: Entries<T, E>,
}

/// Capture this in addon callbacks instead of a strong manager clone to avoid
/// a manager -> addon -> manager ownership cycle.
pub struct WeakAddonManager<T, E> {
    entries: Weak<EntryList<T, E>>,
}
impl<T, E> WeakAddonManager<T, E> {
    pub fn upgrade(&self) -> Option<AddonManager<T, E>> {
        self.entries
            .upgrade()
            .map(|entries| AddonManager { entries })
    }
}
impl<T, E> Default for AddonManager<T, E> {
    fn default() -> Self {
        Self {
            entries: Rc::default(),
        }
    }
}
impl<T, E> Clone for AddonManager<T, E> {
    fn clone(&self) -> Self {
        Self {
            entries: Rc::clone(&self.entries),
        }
    }
}
impl<T, E> AddonManager<T, E> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn downgrade(&self) -> WeakAddonManager<T, E> {
        WeakAddonManager {
            entries: Rc::downgrade(&self.entries),
        }
    }
    pub fn len(&self) -> usize {
        self.entries.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.borrow().is_empty()
    }

    /// Records ownership before activation. Activation failure leaves the entry
    /// registered so later explicit manager disposal can still clean it up.
    pub fn load_addon(
        &self,
        terminal: &T,
        instance: Rc<dyn TerminalAddon<T, Error = E>>,
    ) -> Result<AddonHandle<T, E>, AddonError<E>> {
        let handle = AddonHandle {
            entries: Rc::clone(&self.entries),
            entry: Rc::new(Entry {
                instance,
                disposed: Cell::new(false),
            }),
        };
        self.entries.borrow_mut().push(Rc::clone(&handle.entry));
        handle
            .entry
            .instance
            .activate(terminal)
            .map_err(AddonError::Callback)?;
        Ok(handle)
    }

    pub fn dispose(&self) -> Result<(), AddonError<E>> {
        let count = self.len();
        for index in (0..count).rev() {
            let entry = self
                .entries
                .borrow()
                .get(index)
                .cloned()
                .ok_or(AddonError::TraversalInvalidated)?;
            AddonHandle {
                entries: Rc::clone(&self.entries),
                entry,
            }
            .dispose()?;
        }
        Ok(())
    }
}
