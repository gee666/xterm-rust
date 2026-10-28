//! Explicit cursors permit source-style mutation without a borrowed vector iterator.
use super::{KeyReader, SortedKey, SortedList, SortedListError};
use std::{cell::Cell, rc::Rc};
// One JS module work variable is shared by every list in its UI realm.
// Thread-local ownership models that realm without cross-thread mutable state.
thread_local! { static WORK_INDEX: Rc<Cell<usize>> = Rc::new(Cell::new(0)); }
#[derive(Clone)]
pub struct IterationContext(pub(super) Rc<Cell<usize>>);
impl Default for IterationContext {
    fn default() -> Self {
        Self(WORK_INDEX.with(Rc::clone))
    }
}
impl IterationContext {
    /// A native-only isolation choice, distinct from the shared source default.
    pub fn isolated() -> Self {
        Self(Rc::new(Cell::new(0)))
    }
}

/// A source generator can yield undefined after a nested operation changes its
/// shared work index. This is distinct from iterator completion.
#[derive(Clone, Debug, PartialEq)]
pub enum KeyItem<T> {
    Value(T),
    Undefined,
}
pub struct KeyCursor {
    key: SortedKey,
    started: bool,
    done: bool,
}
impl KeyCursor {
    pub fn new(key: impl Into<SortedKey>) -> Self {
        Self {
            key: key.into(),
            started: false,
            done: false,
        }
    }
}
impl<T, R: KeyReader<T>> SortedList<T, R> {
    pub fn with_iteration_context(mut self, context: IterationContext) -> Self {
        self.iteration_context = context;
        self
    }
    pub fn iteration_context(&self) -> IterationContext {
        self.iteration_context.clone()
    }
    pub fn next_key_item(
        &mut self,
        cursor: &mut KeyCursor,
    ) -> Result<Option<KeyItem<T>>, SortedListError>
    where
        T: Clone,
    {
        if cursor.done {
            return Ok(None);
        }
        let result = self.resume_key_item(cursor);
        if result.is_err() {
            cursor.done = true;
        }
        result
    }
    fn resume_key_item(
        &mut self,
        cursor: &mut KeyCursor,
    ) -> Result<Option<KeyItem<T>>, SortedListError>
    where
        T: Clone,
    {
        if cursor.started {
            let Some(index) = self.iteration_context.0.get().checked_add(1) else {
                cursor.done = true;
                return Ok(None);
            };
            self.iteration_context.0.set(index);
        } else {
            self.flush()?;
            if self.values.is_empty() {
                cursor.done = true;
                return Ok(None);
            }
            let index = self.search(&cursor.key)?;
            self.iteration_context.0.set(index);
            cursor.started = true;
        }
        let index = self.iteration_context.0.get();
        if index >= self.values.len()
            || !self
                .read_key(self.values.get(index))?
                .strict_eq(&cursor.key)
        {
            cursor.done = true;
            return Ok(None);
        }
        // Getter callbacks may have changed the shared index in another list.
        // Source yield reads that new index, not the earlier key-test element.
        Ok(Some(
            match self.values.get(self.iteration_context.0.get()) {
                Some(value) if !self.is_undefined_value(value) => KeyItem::Value(value.clone()),
                _ => KeyItem::Undefined,
            },
        ))
    }
    /// Native typed convenience. The fully faithful next_key_item represents an
    /// out-of-bounds undefined yield even when T cannot contain that primitive.
    pub fn next_by_key(&mut self, cursor: &mut KeyCursor) -> Result<Option<T>, SortedListError>
    where
        T: Clone,
    {
        match self.next_key_item(cursor)? {
            Some(KeyItem::Value(value)) => Ok(Some(value)),
            Some(KeyItem::Undefined) => match self.values.get(self.iteration_context.0.get()) {
                Some(value) if self.is_undefined_value(value) => Ok(Some(value.clone())),
                _ => {
                    cursor.done = true;
                    Err(SortedListError::UndefinedYield)
                }
            },
            None => Ok(None),
        }
    }
    pub fn for_each_by_key_reentrant(
        &mut self,
        key: impl Into<SortedKey>,
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
    pub fn try_for_each_by_key_reentrant<E: From<SortedListError>>(
        &mut self,
        key: impl Into<SortedKey>,
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
    pub fn try_for_each_key_item_reentrant<E: From<SortedListError>>(
        &mut self,
        key: impl Into<SortedKey>,
        mut callback: impl FnMut(&mut Self, KeyItem<T>) -> Result<(), E>,
    ) -> Result<(), E>
    where
        T: Clone,
    {
        let mut cursor = KeyCursor::new(key);
        while let Some(value) = self.next_key_item(&mut cursor)? {
            callback(self, value)?;
        }
        Ok(())
    }
}
