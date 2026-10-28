// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use crate::circular_list::{CircularList, ListChange};
#[path = "decoration_iterator.rs"]
mod iterator;
pub use iterator::DecorationCellIterator;
type Bucket = Rc<RefCell<Vec<Decoration>>>;
type Buckets = Vec<(i64, Bucket)>;
#[derive(Clone, Copy)]
enum Change {
    Insert(ListChange),
    Delete(ListChange),
}
struct CacheState {
    buckets: RefCell<Buckets>,
    decorations: RefCell<Vec<Decoration>>,
    listeners: RefCell<Vec<DisposableHandle>>,
    pending: RefCell<Vec<Change>>,
    sync_requested: Emitter<()>,
    scheduled: Cell<bool>,
    disposed: Cell<bool>,
}
impl Drop for CacheState {
    fn drop(&mut self) {
        for listener in self.listeners.get_mut().drain(..) {
            listener.dispose();
        }
    }
}

/// Logical-line index. Insert/delete work runs only at the host microtask boundary.
#[derive(Clone)]
pub struct DecorationLineCache {
    state: Rc<CacheState>,
}
impl Default for DecorationLineCache {
    fn default() -> Self {
        Self::new()
    }
}
impl DecorationLineCache {
    pub fn new() -> Self {
        Self {
            state: Rc::new(CacheState {
                buckets: RefCell::new(Vec::new()),
                decorations: RefCell::new(Vec::new()),
                listeners: RefCell::new(Vec::new()),
                pending: RefCell::new(Vec::new()),
                sync_requested: Emitter::new(),
                scheduled: Cell::new(false),
                disposed: Cell::new(false),
            }),
        }
    }
    pub fn on_sync_requested(&self) -> Event<()> {
        self.state.sync_requested.event()
    }
    pub fn has_pending_sync(&self) -> bool {
        self.state.scheduled.get()
    }
    pub fn clear(&self) {
        self.state.pending.borrow_mut().clear();
        self.state.scheduled.set(false);
        self.state.buckets.borrow_mut().clear();
        self.state.decorations.borrow_mut().clear();
    }
    pub fn dispose(&self) {
        if self.state.disposed.replace(true) {
            return;
        }
        let listeners = self.state.listeners.take();
        for listener in listeners {
            listener.dispose();
        }
        self.state.pending.borrow_mut().clear();
        self.state.scheduled.set(false);
        self.state.sync_requested.dispose();
    }
    pub fn add(&self, decoration: Decoration) {
        if !self
            .state
            .decorations
            .borrow()
            .iter()
            .any(|d| d.ptr_eq(&decoration))
        {
            self.state.decorations.borrow_mut().push(decoration.clone());
        }
        self.add_to_buckets(&decoration);
    }
    pub fn remove(&self, decoration: &Decoration) {
        self.state
            .decorations
            .borrow_mut()
            .retain(|d| !d.ptr_eq(decoration));
        self.remove_from_buckets(decoration);
    }
    pub fn decorations_on_line(&self, line: i64) -> Option<Vec<Decoration>> {
        self.bucket(line).map(|bucket| bucket.borrow().clone())
    }
    fn bucket(&self, line: i64) -> Option<Bucket> {
        self.state
            .buckets
            .borrow()
            .iter()
            .find(|(key, _)| *key == line)
            .map(|(_, bucket)| Rc::clone(bucket))
    }
    fn add_to_buckets(&self, decoration: &Decoration) {
        let start = decoration.marker().line();
        if start < 0 {
            return;
        }
        decoration.state.indexed_start.set(start);
        for offset in 0..decoration.height() {
            let Some(line) = start.checked_add(offset) else {
                break;
            };
            let bucket = match self.bucket(line) {
                Some(bucket) => bucket,
                None => {
                    let bucket = Rc::new(RefCell::new(Vec::new()));
                    self.state
                        .buckets
                        .borrow_mut()
                        .push((line, Rc::clone(&bucket)));
                    bucket
                }
            };
            bucket.borrow_mut().push(decoration.clone());
        }
    }
    fn remove_from_buckets(&self, decoration: &Decoration) {
        let start = decoration.state.indexed_start.get();
        for offset in 0..decoration.height() {
            let Some(line) = start.checked_add(offset) else {
                break;
            };
            if let Some(bucket) = self.bucket(line) {
                let position = bucket.borrow().iter().position(|d| d.ptr_eq(decoration));
                if let Some(position) = position {
                    bucket.borrow_mut().remove(position);
                }
                if bucket.borrow().is_empty() {
                    self.state
                        .buckets
                        .borrow_mut()
                        .retain(|(key, _)| *key != line);
                }
            }
        }
    }
    pub fn attach_to_buffer_lines<T: Clone + 'static>(&self, lines: &CircularList<T>) {
        // MutableDisposable disposes replacement listener stores after disposal.
        if self.state.disposed.get() {
            return;
        }
        let old = self.state.listeners.take();
        for listener in old {
            listener.dispose();
        }
        let weak = Rc::downgrade(&self.state);
        let trim = lines.on_trim_emitter.event().try_subscribe(move |amount| {
            if let Some(state) = weak.upgrade() {
                DecorationLineCache { state }.trim(*amount)?;
            }
            Ok(())
        });
        let weak = Rc::downgrade(&self.state);
        let insert = lines.on_insert_emitter.event().try_subscribe(move |event| {
            if let Some(state) = weak.upgrade() {
                DecorationLineCache { state }.queue(Change::Insert(*event))?;
            }
            Ok(())
        });
        let weak = Rc::downgrade(&self.state);
        let delete = lines.on_delete_emitter.event().try_subscribe(move |event| {
            if let Some(state) = weak.upgrade() {
                DecorationLineCache { state }.queue(Change::Delete(*event))?;
            }
            Ok(())
        });
        *self.state.listeners.borrow_mut() = vec![trim, insert, delete];
    }
    fn queue(&self, change: Change) -> Result<(), EmitError> {
        self.state.pending.borrow_mut().push(change);
        if self.state.disposed.get() {
            return Err(EmitError::callback(std::io::Error::other(
                "Calling set on a disposed MicrotaskTimer",
            )));
        }
        if !self.state.scheduled.replace(true) {
            self.state.sync_requested.fire(&())?;
        }
        Ok(())
    }
    /// Call from queueMicrotask, after all synchronous marker listeners finish.
    /// A stale scheduled callback after clear/dispose finds no pending work.
    pub fn flush_pending(&self) -> Result<(), EmitError> {
        self.state.scheduled.set(false);
        let pending = self.state.pending.take();
        for change in pending {
            match change {
                Change::Insert(e) => self.insert(e)?,
                Change::Delete(e) => self.delete(e)?,
            }
        }
        Ok(())
    }
    fn remap(&self, map: impl Fn(i64) -> Option<i64>) {
        let mut next: Buckets = Vec::new();
        for (line, bucket) in self.state.buckets.borrow().iter() {
            if let Some(line) = map(*line) {
                if let Some((_, existing)) = next.iter().find(|(key, _)| *key == line) {
                    existing
                        .borrow_mut()
                        .extend(bucket.borrow().iter().cloned());
                } else {
                    next.push((line, Rc::new(RefCell::new(bucket.borrow().clone()))));
                }
            }
        }
        *self.state.buckets.borrow_mut() = next;
    }
    fn trim(&self, amount: usize) -> Result<(), EmitError> {
        if amount == 0 || self.state.buckets.borrow().is_empty() {
            return Ok(());
        }
        let amount = i64::try_from(amount).map_err(EmitError::callback)?;
        self.remap(|line| line.checked_sub(amount).filter(|line| *line >= 0));
        for d in self.state.decorations.borrow().iter() {
            if !d.marker().is_disposed() {
                d.state
                    .indexed_start
                    .set(d.state.indexed_start.get().saturating_sub(amount));
            }
        }
        Ok(())
    }
    fn insert(&self, event: ListChange) -> Result<(), EmitError> {
        let index = i64::try_from(event.index).map_err(EmitError::callback)?;
        let amount = i64::try_from(event.amount).map_err(EmitError::callback)?;
        let decorations = self.state.decorations.borrow().clone();
        let crossers: Vec<_> = decorations
            .iter()
            .filter(|d| {
                !d.marker().is_disposed()
                    && d.state.indexed_start.get() < index
                    && d.state.indexed_start.get().saturating_add(d.height()) > index
            })
            .cloned()
            .collect();
        for d in &crossers {
            self.remove_from_buckets(d);
        }
        self.remap(|line| {
            if line >= index {
                line.checked_add(amount)
            } else {
                Some(line)
            }
        });
        for d in decorations {
            if !d.marker().is_disposed() && d.state.indexed_start.get() >= index {
                d.state.indexed_start.set(d.marker().line());
            }
        }
        for d in crossers {
            self.add_to_buckets(&d);
        }
        Ok(())
    }
    fn delete(&self, event: ListChange) -> Result<(), EmitError> {
        let index = i64::try_from(event.index).map_err(EmitError::callback)?;
        let amount = i64::try_from(event.amount).map_err(EmitError::callback)?;
        let end = index.saturating_add(amount);
        self.remap(|line| {
            if line >= index && line < end {
                None
            } else if line >= end {
                line.checked_sub(amount)
            } else {
                Some(line)
            }
        });
        let decorations = self.state.decorations.borrow().clone();
        for d in decorations {
            if d.marker().is_disposed() {
                continue;
            }
            let start = d.state.indexed_start.get();
            if start >= end {
                d.state.indexed_start.set(d.marker().line());
            } else if start < index && start.saturating_add(d.height()) > end {
                self.remove_from_buckets(&d);
                if d.marker().line() >= 0 {
                    self.add_to_buckets(&d);
                }
            }
        }
        Ok(())
    }
    pub fn at_cell(
        &self,
        x: f64,
        line: i64,
        layer: Option<DecorationLayer>,
    ) -> DecorationCellIterator {
        DecorationCellIterator::new(self.clone(), x, line, layer)
    }
}
