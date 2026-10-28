// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Shared streaming-parser registration ownership.

use crate::lifecycle::{to_disposable, DisposableHandle};
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;
use std::rc::{Rc, Weak};

pub type HandlerHandle<H> = Rc<RefCell<H>>;
pub type HandlerList<H> = Rc<RefCell<Vec<HandlerHandle<H>>>>;
type Entries<K, H> = RefCell<HashMap<K, HandlerList<H>>>;

/// Lists stay shared with an active parser after lookup removal. Callbacks may
/// mutate registration without retaining a borrow of the map or active list.
pub struct HandlerRegistry<K, H: ?Sized> {
    entries: Rc<Entries<K, H>>,
}

impl<K, H: ?Sized> Default for HandlerRegistry<K, H> {
    fn default() -> Self {
        Self {
            entries: Rc::new(RefCell::new(HashMap::new())),
        }
    }
}

impl<K: Eq + Hash, H: ?Sized + 'static> HandlerRegistry<K, H> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, key: K, handler: HandlerHandle<H>) -> DisposableHandle {
        register(&self.entries, key, handler)
    }

    pub fn clear(&self, key: &K) {
        self.entries.borrow_mut().remove(key);
    }

    pub fn clear_all(&self) {
        self.entries.borrow_mut().clear();
    }

    pub fn get(&self, key: &K) -> HandlerList<H> {
        self.entries.borrow().get(key).cloned().unwrap_or_default()
    }

    pub fn registrar(&self) -> Registrar<K, H> {
        Registrar {
            entries: Rc::downgrade(&self.entries),
        }
    }
}

/// Callback-safe registration handle. It does not keep its parser or registered
/// handlers alive. Explicit parser disposal leaves a live registry reusable.
pub struct Registrar<K, H: ?Sized> {
    entries: Weak<Entries<K, H>>,
}

impl<K, H: ?Sized> Clone for Registrar<K, H> {
    fn clone(&self) -> Self {
        Self {
            entries: self.entries.clone(),
        }
    }
}

impl<K: Eq + Hash, H: ?Sized + 'static> Registrar<K, H> {
    /// None means the owning parser was dropped, not merely explicitly disposed.
    pub fn register(&self, key: K, handler: HandlerHandle<H>) -> Option<DisposableHandle> {
        Some(register(&self.entries.upgrade()?, key, handler))
    }

    /// Clearing an absent key succeeds while the owning parser remains alive.
    pub fn clear(&self, key: &K) -> Option<()> {
        self.entries.upgrade()?.borrow_mut().remove(key);
        Some(())
    }
}

fn register<K: Eq + Hash, H: ?Sized + 'static>(
    entries: &Rc<Entries<K, H>>,
    key: K,
    handler: HandlerHandle<H>,
) -> DisposableHandle {
    let list = Rc::clone(entries.borrow_mut().entry(key).or_default());
    list.borrow_mut().push(Rc::clone(&handler));
    let list = Rc::downgrade(&list);
    let handler = Rc::downgrade(&handler);
    to_disposable(move || {
        // Weak captures prevent callback -> own token -> callback cycles. An
        // active list or registry, not its token, owns callback lifetime.
        let (Some(list), Some(handler)) = (list.upgrade(), handler.upgrade()) else {
            return;
        };
        let mut list = list.borrow_mut();
        if let Some(index) = list.iter().position(|entry| Rc::ptr_eq(entry, &handler)) {
            list.remove(index);
        }
    })
}
