// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use xterm_core::lifecycle::Disposable;

type Providers<T> = Rc<RefCell<Vec<Rc<T>>>>;

/// Registration order determines priority. Use `snapshot` before calling providers
/// so reentrant registration/disposal never runs under a RefCell borrow.
pub struct LinkProviderService<T: ?Sized> {
    providers: Providers<T>,
}

impl<T: ?Sized> Default for LinkProviderService<T> {
    fn default() -> Self {
        Self {
            providers: Rc::new(RefCell::new(Vec::new())),
        }
    }
}

impl<T: ?Sized> LinkProviderService<T> {
    pub fn register(&self, provider: Rc<T>) -> ProviderRegistration<T> {
        self.providers.borrow_mut().push(Rc::clone(&provider));
        ProviderRegistration {
            providers: Rc::downgrade(&self.providers),
            provider,
        }
    }

    pub fn snapshot(&self) -> Vec<Rc<T>> {
        self.providers.borrow().clone()
    }
    /// Live iteration can observe providers appended by an earlier callback.
    pub fn get(&self, index: usize) -> Option<Rc<T>> {
        self.providers.borrow().get(index).cloned()
    }
    pub fn len(&self) -> usize {
        self.providers.borrow().len()
    }
    pub fn is_empty(&self) -> bool {
        self.providers.borrow().is_empty()
    }
}

impl<T: ?Sized> Disposable for LinkProviderService<T> {
    fn dispose(&self) {
        // Release the borrow before dropping providers, whose destructors can reenter.
        let removed = std::mem::take(&mut *self.providers.borrow_mut());
        drop(removed);
    }
}

impl<T: ?Sized> Drop for LinkProviderService<T> {
    fn drop(&mut self) {
        self.dispose();
    }
}

/// Explicit disposal follows the source's indexOf behavior: every call removes
/// the first matching identity, including a second registration of the same object.
/// Dropping this handle alone does not dispose it, matching core lifecycle handles.
pub struct ProviderRegistration<T: ?Sized> {
    providers: Weak<RefCell<Vec<Rc<T>>>>,
    provider: Rc<T>,
}

impl<T: ?Sized> Disposable for ProviderRegistration<T> {
    fn dispose(&self) {
        let Some(providers) = self.providers.upgrade() else {
            return;
        };
        let removed = {
            let mut entries = providers.borrow_mut();
            entries
                .iter()
                .position(|entry| Rc::ptr_eq(entry, &self.provider))
                .map(|index| entries.remove(index))
        };
        drop(removed);
    }
}
