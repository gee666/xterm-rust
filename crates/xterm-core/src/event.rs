// Copyright (c) 2024-2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Adapted from common/Event.ts.

//! Synchronous, single-threaded events. Capture a receiver in the listener closure
//! instead of JavaScript's `thisArgs`. Subscriptions require explicit disposal,
//! so discarding a handle does not unregister its listener.

use crate::lifecycle::{to_disposable, Disposable, DisposableHandle, DisposableStore};
use std::cell::{Cell, RefCell};
use std::fmt;
use std::rc::Rc;

/// Upstream clears the current listener array when an emitter is disposed. If
/// that array is being traversed, accessing its next entry throws a TypeError.
#[derive(Clone, Debug)]
pub enum EmitError {
    ListenerArrayCleared,
    /// Retain an actual callback failure, including a browser-owned wrapper
    /// around a thrown JsValue. The portable core does not depend on that type.
    Callback(Rc<dyn std::error::Error>),
}

impl EmitError {
    pub fn callback(error: impl std::error::Error + 'static) -> Self {
        Self::Callback(Rc::new(error))
    }

    /// Recover the exact callback error object without string conversion.
    pub fn callback_ref<E: std::error::Error + 'static>(&self) -> Option<&E> {
        match self {
            Self::Callback(error) => error.downcast_ref(),
            Self::ListenerArrayCleared => None,
        }
    }
}

impl PartialEq for EmitError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::ListenerArrayCleared, Self::ListenerArrayCleared) => true,
            (Self::Callback(left), Self::Callback(right)) => Rc::ptr_eq(left, right),
            _ => false,
        }
    }
}
impl Eq for EmitError {}

impl fmt::Display for EmitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ListenerArrayCleared => {
                f.write_str("listener array cleared during event dispatch")
            }
            Self::Callback(error) => fmt::Display::fmt(error, f),
        }
    }
}

impl std::error::Error for EmitError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Callback(error) => Some(error.as_ref()),
            Self::ListenerArrayCleared => None,
        }
    }
}

type Callback<T> = Rc<dyn Fn(&T) -> Result<(), EmitError>>;
type Listeners<T> = Rc<RefCell<Vec<Callback<T>>>>;
type Subscribe<T> = dyn Fn(Callback<T>) -> DisposableHandle;

/// A callable event registration contract, including mapped or combined events.
pub struct Event<T> {
    subscribe: Rc<Subscribe<T>>,
}

impl<T> Clone for Event<T> {
    fn clone(&self) -> Self {
        Self {
            subscribe: Rc::clone(&self.subscribe),
        }
    }
}

impl<T: 'static> Event<T> {
    /// Registration-function identity, corresponding to JavaScript `===`.
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.subscribe, &other.subscribe)
    }

    pub fn subscribe(&self, listener: impl Fn(&T) + 'static) -> DisposableHandle {
        self.try_subscribe(move |value| {
            listener(value);
            Ok(())
        })
    }

    /// Fallible listeners propagate another emitter's synchronous dispatch error.
    pub fn try_subscribe(
        &self,
        listener: impl Fn(&T) -> Result<(), EmitError> + 'static,
    ) -> DisposableHandle {
        (self.subscribe)(Rc::new(listener))
    }

    /// The returned handle and the store refer to the same subscription.
    pub fn subscribe_in(
        &self,
        store: &DisposableStore,
        listener: impl Fn(&T) + 'static,
    ) -> DisposableHandle {
        store.add(self.subscribe(listener))
    }

    /// JavaScript's disposable array is represented by an owned handle vector.
    pub fn subscribe_in_vec(
        &self,
        disposables: &mut Vec<DisposableHandle>,
        listener: impl Fn(&T) + 'static,
    ) -> DisposableHandle {
        let handle = self.subscribe(listener);
        disposables.push(Rc::clone(&handle));
        handle
    }

    pub fn map<O: 'static>(&self, map: impl Fn(&T) -> O + 'static) -> Event<O> {
        let source = self.clone();
        let map = Rc::new(map);
        Event {
            subscribe: Rc::new(move |listener| {
                let map = Rc::clone(&map);
                source.try_subscribe(move |value| listener(&map(value)))
            }),
        }
    }

    pub fn forward(&self, target: Rc<Emitter<T>>) -> DisposableHandle {
        self.try_subscribe(move |value| target.fire(value))
    }

    /// Invoke the handler before registering, matching runAndSubscribe ordering.
    /// `None` represents the overload without an initial value.
    pub fn run_and_subscribe(
        &self,
        handler: impl Fn(Option<&T>) + 'static,
        initial: Option<&T>,
    ) -> DisposableHandle {
        handler(initial);
        self.subscribe(move |value| handler(Some(value)))
    }

    pub fn any(events: impl IntoIterator<Item = Event<T>>) -> Self {
        let events: Vec<_> = events.into_iter().collect();
        Self {
            subscribe: Rc::new(move |listener| {
                let store = Rc::new(DisposableStore::new_explicit());
                for event in &events {
                    store.add((event.subscribe)(Rc::clone(&listener)));
                }
                store
            }),
        }
    }
}

struct State<T> {
    listeners: RefCell<Listeners<T>>,
    disposed: Cell<bool>,
}

/// Listener additions/removals copy the array, so an active fire sees its original
/// listeners. Recursive fires see the current array. No borrow crosses callbacks.
pub struct Emitter<T> {
    state: Rc<State<T>>,
    event: Event<T>,
}

impl<T: 'static> Default for Emitter<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: 'static> Emitter<T> {
    pub fn new() -> Self {
        let state = Rc::new(State {
            listeners: RefCell::new(Rc::new(RefCell::new(Vec::new()))),
            disposed: Cell::new(false),
        });
        // The cached event owns state, but state does not own the event. This
        // preserves getter identity and retained events without a reference cycle.
        Self {
            state: Rc::clone(&state),
            event: Self::registration(state),
        }
    }

    pub fn event(&self) -> Event<T> {
        self.event.clone()
    }

    fn registration(state: Rc<State<T>>) -> Event<T> {
        Event {
            subscribe: Rc::new(move |listener| {
                if state.disposed.get() {
                    return Rc::new(crate::lifecycle::NoopDisposable);
                }
                let mut listeners = state.listeners.borrow().borrow().clone();
                listeners.push(Rc::clone(&listener));
                *state.listeners.borrow_mut() = Rc::new(RefCell::new(listeners));
                let weak = Rc::downgrade(&state);
                to_disposable(move || {
                    if let Some(state) = weak.upgrade() {
                        let mut listeners = state.listeners.borrow().borrow().clone();
                        if let Some(index) = listeners
                            .iter()
                            .position(|entry| Rc::ptr_eq(entry, &listener))
                        {
                            listeners.remove(index);
                            *state.listeners.borrow_mut() = Rc::new(RefCell::new(listeners));
                        }
                    }
                })
            }),
        }
    }

    /// Borrowing the value avoids cloning shared event payloads.
    pub fn fire(&self, value: &T) -> Result<(), EmitError> {
        if self.state.disposed.get() {
            return Ok(());
        }
        let listeners = Rc::clone(&self.state.listeners.borrow());
        let length = listeners.borrow().len();
        for index in 0..length {
            let listener = listeners
                .borrow()
                .get(index)
                .cloned()
                .ok_or(EmitError::ListenerArrayCleared)?;
            listener(value)?;
        }
        Ok(())
    }
}

impl<T> Disposable for Emitter<T> {
    fn dispose(&self) {
        if !self.state.disposed.replace(true) {
            // Captured Rust resources can run Drop code that disposes another
            // subscription. Empty the same array first, then release callbacks
            // outside both RefCell borrows without changing snapshot semantics.
            let removed = {
                let listeners = self.state.listeners.borrow();
                let mut entries = listeners.borrow_mut();
                std::mem::take(&mut *entries)
            };
            drop(removed);
        }
    }
}
