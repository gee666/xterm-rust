// Copyright (c) 2024-2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Adapted from src/common/Lifecycle.ts.

//! Single-threaded lifecycle contracts matching xterm.js callback ordering.
//!
//! Handles use `Rc` identity, like JavaScript object identity. Disposal is explicit
//! for handles and repeatable unless the underlying implementation is idempotent.
//! Stores and mutable owners also dispose on drop, Rust's resource cleanup boundary.
//! Callbacks run without a borrowed store, so callbacks may register other handles.

use std::cell::{Cell, RefCell};
use std::rc::Rc;

/// An explicitly disposable resource. Implementations decide whether calls repeat.
pub trait Disposable {
    fn dispose(&self);
}

/// Shared object identity for a disposable, without requiring thread safety.
pub type DisposableHandle = Rc<dyn Disposable>;

struct CallbackDisposable<F: Fn()>(F);

impl<F: Fn()> Disposable for CallbackDisposable<F> {
    fn dispose(&self) {
        (self.0)();
    }
}

/// Wrap a callback. As upstream, every disposal invokes the callback again.
pub fn to_disposable(callback: impl Fn() + 'static) -> DisposableHandle {
    Rc::new(CallbackDisposable(callback))
}

/// Dispose a resource and return the same handle.
pub fn dispose(resource: DisposableHandle) -> DisposableHandle {
    resource.dispose();
    resource
}

/// Rust representation of upstream's optional `dispose` overload.
pub fn dispose_optional(resource: Option<DisposableHandle>) -> Option<DisposableHandle> {
    if let Some(resource) = &resource {
        resource.dispose();
    }
    resource
}

/// Dispose in insertion order and return an empty list, as upstream's array overload.
pub fn dispose_all(resources: Vec<DisposableHandle>) -> Vec<DisposableHandle> {
    for resource in resources {
        resource.dispose();
    }
    Vec::new()
}

/// Group disposables without deduplication. Each disposal repeats the whole group.
pub fn combined_disposable(resources: Vec<DisposableHandle>) -> DisposableHandle {
    to_disposable(move || {
        for resource in &resources {
            resource.dispose();
        }
    })
}

/// Upstream's `Disposable.None`, an intentionally empty disposable resource.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoopDisposable;

impl Disposable for NoopDisposable {
    fn dispose(&self) {}
}

pub const NONE: NoopDisposable = NoopDisposable;

/// Own resources in registration order, deduplicated by object identity.
///
/// `clear` permits reuse. `dispose` permanently closes the store and disposes any
/// subsequently added resource immediately. Registration during `clear` participates
/// in that same traversal, matching JavaScript Set iteration.
#[derive(Default)]
pub struct DisposableStore {
    resources: RefCell<StoreResources>,
    disposed: Cell<bool>,
    explicit_disposal: bool,
}

#[derive(Default)]
struct StoreResources {
    entries: Vec<Option<DisposableHandle>>,
    iterators: usize,
}

// JavaScript Set.clear leaves tombstones for active iterators. Keep those slots
// until every traversal ends, including when a callback unwinds through one.
struct StoreIteration<'a>(&'a RefCell<StoreResources>);

impl Drop for StoreIteration<'_> {
    fn drop(&mut self) {
        let mut resources = self.0.borrow_mut();
        resources.iterators -= 1;
        if resources.iterators == 0 {
            resources.entries.retain(Option::is_some);
        }
    }
}

impl DisposableStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Do not call disposal callbacks on drop. Event subscription groups need
    /// JavaScript's explicit-only lifetime: ignoring a returned registration
    /// handle must not unsubscribe it. Stored Rust handles still drop normally.
    pub fn new_explicit() -> Self {
        Self {
            resources: RefCell::default(),
            disposed: Cell::new(false),
            explicit_disposal: true,
        }
    }

    pub fn is_disposed(&self) -> bool {
        self.disposed.get()
    }

    /// Retain and return the exact same resource object.
    pub fn add(&self, resource: DisposableHandle) -> DisposableHandle {
        if self.is_disposed() {
            resource.dispose();
        } else {
            let mut resources = self.resources.borrow_mut();
            if !resources
                .entries
                .iter()
                .flatten()
                .any(|entry| Rc::ptr_eq(entry, &resource))
            {
                resources.entries.push(Some(Rc::clone(&resource)));
            }
        }
        resource
    }

    pub fn clear(&self) {
        self.resources.borrow_mut().iterators += 1;
        let _iteration = StoreIteration(&self.resources);
        let mut index = 0;
        loop {
            let entry = self.resources.borrow().entries.get(index).cloned();
            match entry {
                None => break,
                Some(None) => {}
                Some(Some(resource)) => resource.dispose(),
            }
            index += 1;
        }
        let removed: Vec<_> = self
            .resources
            .borrow_mut()
            .entries
            .iter_mut()
            .filter_map(Option::take)
            .collect();
        drop(removed);
    }
}

impl Disposable for DisposableStore {
    fn dispose(&self) {
        if !self.disposed.replace(true) {
            self.clear();
        }
    }
}

impl Drop for DisposableStore {
    fn drop(&mut self) {
        if !self.explicit_disposal {
            self.dispose();
        }
    }
}

/// Rust composition replacement for upstream's abstract `Disposable` base class.
///
/// Embed this owner and forward `Disposable::dispose` to it. `register` replaces
/// the protected `_register` method without requiring inheritance.
#[derive(Default)]
pub struct DisposableOwner {
    store: DisposableStore,
}

impl DisposableOwner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, resource: DisposableHandle) -> DisposableHandle {
        self.store.add(resource)
    }
}

impl Disposable for DisposableOwner {
    fn dispose(&self) {
        self.store.dispose();
    }
}

/// Own one replaceable resource, disposing the old one before storing a new one.
///
/// Assignment after disposal is ignored without disposing the assigned object,
/// matching upstream. Getters hide the current value during disposal callbacks.
#[derive(Default)]
pub struct MutableDisposable {
    value: RefCell<Option<DisposableHandle>>,
    disposed: Cell<bool>,
}

impl MutableDisposable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(&self) -> Option<DisposableHandle> {
        if self.disposed.get() {
            None
        } else {
            self.value.borrow().clone()
        }
    }

    pub fn set(&self, value: Option<DisposableHandle>) {
        if self.disposed.get() {
            return;
        }
        let old = self.value.borrow().clone();
        let same = match (&old, &value) {
            (Some(old), Some(value)) => Rc::ptr_eq(old, value),
            (None, None) => true,
            _ => false,
        };
        if same {
            return;
        }
        if let Some(old) = old {
            old.dispose();
        }
        let previous = self.value.replace(value);
        drop(previous);
    }

    pub fn clear(&self) {
        self.set(None);
    }
}

impl Disposable for MutableDisposable {
    fn dispose(&self) {
        self.disposed.set(true);
        let old = self.value.borrow().clone();
        if let Some(old) = old {
            old.dispose();
        }
        let previous = self.value.take();
        drop(previous);
    }
}

impl Drop for MutableDisposable {
    fn drop(&mut self) {
        self.dispose();
    }
}
