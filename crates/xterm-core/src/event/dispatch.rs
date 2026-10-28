// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
//! Dispatch-local conversion cache. The core stores no browser-specific types.
use super::EmitError;
use std::{any::Any, cell::RefCell, fmt, marker::PhantomData, rc::Rc};

type Entry = (Rc<()>, Rc<dyn Any>);

/// One converter's memo key. Clone this key across its listener registrations.
/// Different converters use different keys, even when their result types match.
/// A key cannot be reinterpreted through function-pointer subtyping.
///
/// ```compile_fail
/// use xterm_core::event::DispatchKey;
/// type General = fn(&());
/// type Static = fn(&'static ());
/// fn reinterpret(key: DispatchKey<General>) -> DispatchKey<Static> { key }
/// ```
pub struct DispatchKey<T> {
    identity: Rc<()>,
    // Invariance prevents function-pointer/lifetime subtyping from reusing a
    // cached key identity with a different Any TypeId.
    payload: PhantomData<fn(T) -> T>,
}
impl<T> DispatchKey<T> {
    pub fn new() -> Self {
        Self {
            identity: Rc::new(()),
            payload: PhantomData,
        }
    }
}
impl<T> Default for DispatchKey<T> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T> Clone for DispatchKey<T> {
    fn clone(&self) -> Self {
        Self {
            identity: Rc::clone(&self.identity),
            payload: PhantomData,
        }
    }
}

/// Borrowed for one synchronous dispatch. Attachments are released when that
/// dispatch returns or unwinds; a listener may explicitly retain its Rc value.
/// This context cannot be cloned or constructed by listeners, so it cannot be
/// captured by its own static attachments. Nested fires get independent caches.
pub struct DispatchContext {
    // Event converters normally share one key. A short vector avoids hashing
    // and a separate identity map for this small, dispatch-bounded collection.
    values: RefCell<Vec<Entry>>,
}
impl DispatchContext {
    pub(super) fn new() -> Self {
        Self {
            values: RefCell::new(Vec::new()),
        }
    }

    pub fn memoize_with<T: 'static>(
        &self,
        key: &DispatchKey<T>,
        create: impl FnOnce() -> T,
    ) -> Result<Rc<T>, EmitError> {
        self.try_memoize_with(key, || Ok(create()))
    }

    /// A failed conversion is not cached. No cache borrow spans the converter,
    /// which may synchronously trigger nested events or create another value.
    pub fn try_memoize_with<T: 'static>(
        &self,
        key: &DispatchKey<T>,
        create: impl FnOnce() -> Result<T, EmitError>,
    ) -> Result<Rc<T>, EmitError> {
        let existing = find(&self.values.borrow(), &key.identity);
        if let Some(value) = existing {
            return recover(value);
        }
        let created: Rc<dyn Any> = Rc::new(create()?);
        let selected = {
            let mut values = self.values.borrow_mut();
            // Recursive creation of this key keeps its first completed value.
            // Drop any redundant value outside the cache borrow.
            if let Some(value) = find(&values, &key.identity) {
                value
            } else {
                values.try_reserve(1).map_err(EmitError::callback)?;
                values.push((Rc::clone(&key.identity), Rc::clone(&created)));
                Rc::clone(&created)
            }
        };
        recover(selected)
    }
}

fn find(values: &[Entry], key: &Rc<()>) -> Option<Rc<dyn Any>> {
    values
        .iter()
        .find(|(identity, _)| Rc::ptr_eq(identity, key))
        .map(|(_, value)| Rc::clone(value))
}

#[derive(Debug)]
struct MemoTypeMismatch;
impl fmt::Display for MemoTypeMismatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("dispatch memo key does not match its retained payload type")
    }
}
impl std::error::Error for MemoTypeMismatch {}
fn recover<T: 'static>(value: Rc<dyn Any>) -> Result<Rc<T>, EmitError> {
    // Private invariant generic keys cannot change type through the public API.
    // Still report an internal inconsistency instead of panicking in a binding.
    value
        .downcast()
        .map_err(|_| EmitError::callback(MemoTypeMismatch))
}
