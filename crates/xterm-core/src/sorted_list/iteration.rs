//! Explicit cursors allow the source's callback/iterator mutation in safe Rust.
use super::{SortedList, SortedListError};
use std::cell::Cell;
use std::rc::Rc;

/// The reference shares one work index across all SortedList instances. A
/// terminal can share this context between its lists without global state.
#[derive(Clone, Default)]
pub struct IterationContext(pub(super) Rc<Cell<usize>>);

/// A suspended key iterator. Pass it back to [`SortedList::next_by_key`] after
/// mutating a list. Entries are cloned individually, not snapshotted in bulk.
pub struct KeyCursor {
    key: f64,
    started: bool,
    done: bool,
}

impl KeyCursor {
    pub fn new(key: f64) -> Self {
        Self {
            key,
            started: false,
            done: false,
        }
    }
}

impl<T, F: Fn(&T) -> f64> SortedList<T, F> {
    pub fn with_iteration_context(mut self, context: IterationContext) -> Self {
        self.iteration_context = context;
        self
    }

    pub fn iteration_context(&self) -> IterationContext {
        self.iteration_context.clone()
    }

    /// Resume a key iterator, flushing deferred changes only on its first call.
    /// Nested cursors share the context's work index just like the source.
    pub fn next_by_key(&mut self, cursor: &mut KeyCursor) -> Result<Option<T>, SortedListError>
    where
        T: Clone,
    {
        if cursor.done {
            return Ok(None);
        }
        if cursor.started {
            let Some(index) = self.iteration_context.0.get().checked_add(1) else {
                cursor.done = true;
                return Ok(None);
            };
            self.iteration_context.0.set(index);
        } else {
            if let Err(error) = self.flush() {
                // A JavaScript generator closes permanently when its body
                // throws, even if the list later recovers from that error.
                cursor.done = true;
                return Err(error);
            }
            if self.values.is_empty() {
                cursor.done = true;
                return Ok(None);
            }
            self.iteration_context.0.set(self.search(cursor.key));
            cursor.started = true;
        }
        let value = self.values.get(self.iteration_context.0.get());
        if let Some(value) = value.filter(|value| (self.key)(value) == cursor.key) {
            return Ok(Some(value.clone()));
        }
        cursor.done = true;
        Ok(None)
    }

    /// Source-compatible mutation during callbacks. The callback receives the
    /// list explicitly so safe Rust does not need aliasing or a RefCell borrow
    /// held over user code. Rc values preserve JavaScript object identity.
    pub fn for_each_by_key_reentrant(
        &mut self,
        key: f64,
        mut callback: impl FnMut(&mut Self, T),
    ) -> Result<(), SortedListError>
    where
        T: Clone,
    {
        self.try_for_each_by_key_reentrant(key, |list, value| {
            callback(list, value);
            Ok(())
        })
    }

    /// Fallible callbacks stop before the next index increment, retaining the
    /// mutation and work-index state committed before the callback failed.
    pub fn try_for_each_by_key_reentrant<E: From<SortedListError>>(
        &mut self,
        key: f64,
        mut callback: impl FnMut(&mut Self, T) -> Result<(), E>,
    ) -> Result<(), E>
    where
        T: Clone,
    {
        let mut cursor = KeyCursor::new(key);
        while let Some(value) = self.next_by_key(&mut cursor)? {
            callback(self, value)?;
        }
        Ok(())
    }
}
