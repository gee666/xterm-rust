// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of addon-webgl/src/CharAtlasCache.ts.

use super::config::{config_equals, AtlasConfig, MissingAnsiColor};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

/// Explicit disposal may run user events and fail. Physical GPU resources also
/// have their own RAII owner; cache drop must not invent source disposal events.
pub trait AtlasResource {
    type Error;
    fn dispose(&self) -> Result<(), Self::Error>;
}

#[derive(Debug)]
pub enum CacheError<E> {
    Config(MissingAnsiColor),
    Create(E),
    Dispose(E),
}

struct Entry<T: ?Sized, A: ?Sized> {
    atlas: Rc<A>,
    config: Rc<AtlasConfig>,
    owners: RefCell<Vec<Weak<T>>>,
}
type Entries<T, A> = RefCell<Vec<Rc<Entry<T, A>>>>;

/// Explicitly owned shared pool. Weak terminal identities avoid the source's
/// module-global strong-terminal retention while preserving explicit release.
/// The same pool must be passed to all addons in one browser binding runtime.
pub struct AtlasCache<T: ?Sized, A: AtlasResource + ?Sized> {
    entries: Entries<T, A>,
}
impl<T: ?Sized, A: AtlasResource + ?Sized> Default for AtlasCache<T, A> {
    fn default() -> Self {
        Self {
            entries: RefCell::new(Vec::new()),
        }
    }
}
impl<T: ?Sized, A: AtlasResource + ?Sized> AtlasCache<T, A> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Factory and disposal run without a pool/list borrow, permitting callback
    /// reentry. Remove by captured live index after disposal like source splice.
    pub fn acquire(
        &self,
        terminal: &Rc<T>,
        config: AtlasConfig,
        create: impl FnOnce(Rc<AtlasConfig>) -> Result<Rc<A>, A::Error>,
    ) -> Result<Rc<A>, CacheError<A::Error>> {
        let owner = Rc::downgrade(terminal);
        let mut index = 0;
        loop {
            let entry = self.entries.borrow().get(index).cloned();
            let Some(entry) = entry else {
                break;
            };
            let owner_index = entry
                .owners
                .borrow()
                .iter()
                .position(|old| Weak::ptr_eq(old, &owner));
            if let Some(owner_index) = owner_index {
                if config_equals(&entry.config, &config).map_err(CacheError::Config)? {
                    return Ok(Rc::clone(&entry.atlas));
                }
                if entry.owners.borrow().len() == 1 {
                    entry.atlas.dispose().map_err(CacheError::Dispose)?;
                    self.remove_index(index);
                } else {
                    entry.owners.borrow_mut().remove(owner_index);
                }
                break;
            }
            index += 1;
        }
        let mut index = 0;
        loop {
            let entry = self.entries.borrow().get(index).cloned();
            let Some(entry) = entry else {
                break;
            };
            if config_equals(&entry.config, &config).map_err(CacheError::Config)? {
                entry.owners.borrow_mut().push(owner);
                return Ok(Rc::clone(&entry.atlas));
            }
            index += 1;
        }
        let config = Rc::new(config);
        let atlas = create(Rc::clone(&config)).map_err(CacheError::Create)?;
        self.entries.borrow_mut().push(Rc::new(Entry {
            atlas: Rc::clone(&atlas),
            config,
            owners: RefCell::new(vec![owner]),
        }));
        Ok(atlas)
    }

    /// A Weak identity remains valid for release after terminal-first teardown.
    pub fn remove_terminal(&self, terminal: &Weak<T>) -> Result<(), CacheError<A::Error>> {
        let mut index = 0;
        loop {
            let entry = self.entries.borrow().get(index).cloned();
            let Some(entry) = entry else {
                break;
            };
            let owner_index = entry
                .owners
                .borrow()
                .iter()
                .position(|old| Weak::ptr_eq(old, terminal));
            if let Some(owner_index) = owner_index {
                if entry.owners.borrow().len() == 1 {
                    entry.atlas.dispose().map_err(CacheError::Dispose)?;
                    self.remove_index(index);
                } else {
                    entry.owners.borrow_mut().remove(owner_index);
                }
                break;
            }
            index += 1;
        }
        Ok(())
    }

    fn remove_index(&self, index: usize) {
        let mut entries = self.entries.borrow_mut();
        // JS Array.splice beyond the current length deletes nothing. Reentrant
        // disposal may already have changed this live list before we get here.
        if index < entries.len() {
            entries.remove(index);
        }
    }
}
