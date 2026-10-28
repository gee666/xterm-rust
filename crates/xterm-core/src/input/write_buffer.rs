// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Reentrant FIFO terminal writes with host-controlled zero-delay scheduling.
use crate::{
    event::{EmitError, Emitter, Event},
    lifecycle::Disposable,
};
use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    future::Future,
    pin::Pin,
    rc::{Rc, Weak},
};

#[path = "write_buffer_drain.rs"]
mod drain;
#[path = "write_buffer_source.rs"]
mod source;
#[path = "write_buffer_types.rs"]
mod types;
pub use types::{WriteAction, WriteData, WriteError, WritePayload, WriteScheduler};

pub const DISCARD_WATERMARK: usize = 50_000_000;
pub const WRITE_TIMEOUT_MS: f64 = 12.0;
const LENGTH_THRESHOLD: usize = 50;
const SYNC_OFFSET: usize = 0x7fff_ffff;
type Action<S> =
    dyn Fn(&WriteBuffer<S>, Rc<WriteData>, Option<bool>) -> Result<WriteAction, WriteError>;
pub type WriteCallback<S> = Rc<dyn Fn(&WriteBuffer<S>) -> Result<(), WriteError>>;

struct State<S: WriteScheduler> {
    data: RefCell<VecDeque<Rc<WriteData>>>,
    callbacks: RefCell<VecDeque<Option<WriteCallback<S>>>>,
    pending_units: Cell<f64>,
    offset: Cell<usize>,
    sync_writing: Cell<bool>,
    sync_calls: Cell<f64>,
    user_input: Cell<bool>,
    disposed: Cell<bool>,
    timer: RefCell<Option<S::Handle>>,
    scheduler: S,
    action: Rc<Action<S>>,
    parsed: Emitter<()>,
}
impl<S: WriteScheduler> Drop for State<S> {
    fn drop(&mut self) {
        if let Some(handle) = self.timer.get_mut().take() {
            if let Err(error) = self.scheduler.cancel(handle) {
                self.scheduler.report_error(error);
            }
        }
    }
}

/// Cloneable, single-threaded queue handle. Callbacks receive this handle so
/// reentrant writes do not require a captured strong owner or a held borrow.
/// Host requests and futures must never execute inline in scheduler methods.
pub struct WriteBuffer<S: WriteScheduler> {
    state: Rc<State<S>>,
}
pub struct WeakWriteBuffer<S: WriteScheduler> {
    state: Weak<State<S>>,
}
impl<S: WriteScheduler> Clone for WriteBuffer<S> {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }
}
impl<S: WriteScheduler> Clone for WeakWriteBuffer<S> {
    fn clone(&self) -> Self {
        Self {
            state: Weak::clone(&self.state),
        }
    }
}
impl<S: WriteScheduler> WeakWriteBuffer<S> {
    pub fn upgrade(&self) -> Option<WriteBuffer<S>> {
        self.state.upgrade().map(|state| WriteBuffer { state })
    }
}
impl<S: WriteScheduler + 'static> WriteBuffer<S> {
    pub fn new(
        scheduler: S,
        action: impl Fn(&Self, &WriteData, Option<bool>) -> Result<WriteAction, WriteError> + 'static,
    ) -> Self {
        Self::new_owned(scheduler, move |queue, data, result| {
            action(queue, data.as_ref(), result)
        })
    }
    /// Owning action form for parsers that retain the original input across
    /// await. The queue shares its existing Rc rather than cloning a whole Vec.
    pub fn new_owned(
        scheduler: S,
        action: impl Fn(&Self, Rc<WriteData>, Option<bool>) -> Result<WriteAction, WriteError> + 'static,
    ) -> Self {
        Self {
            state: Rc::new(State {
                data: RefCell::new(VecDeque::new()),
                callbacks: RefCell::new(VecDeque::new()),
                pending_units: Cell::new(0.0),
                offset: Cell::new(0),
                sync_writing: Cell::new(false),
                sync_calls: Cell::new(0.0),
                user_input: Cell::new(false),
                disposed: Cell::new(false),
                timer: RefCell::new(None),
                scheduler,
                action: Rc::new(action),
                parsed: Emitter::new(),
            }),
        }
    }
    pub fn downgrade(&self) -> WeakWriteBuffer<S> {
        WeakWriteBuffer {
            state: Rc::downgrade(&self.state),
        }
    }
    pub fn on_write_parsed(&self) -> Event<()> {
        self.state.parsed.event()
    }
    pub fn is_disposed(&self) -> bool {
        self.state.disposed.get()
    }
    pub fn pending_units(&self) -> f64 {
        self.state.pending_units.get()
    }
    pub fn buffer_offset(&self) -> usize {
        self.state.offset.get()
    }
    /// Includes already-processed chunks retained until compaction, as upstream.
    pub fn buffered_chunks(&self) -> usize {
        self.state.data.borrow().len()
    }
    pub fn has_scheduled_write(&self) -> bool {
        self.state.timer.borrow().is_some()
    }
    pub fn handle_user_input(&self) {
        self.state.user_input.set(true);
    }
    pub fn write(
        &self,
        data: WriteData,
        callback: Option<WriteCallback<S>>,
    ) -> Result<(), WriteError> {
        if self.is_disposed() {
            return Ok(());
        }
        if self.pending_units() > DISCARD_WATERMARK as f64 {
            return Err(WriteError::Discarded);
        }
        let empty = self.state.data.borrow().is_empty();
        if empty {
            self.state.offset.set(0);
            if self.state.user_input.replace(false) {
                self.enqueue(data, callback)?;
                return self.inner_write(0.0, true);
            }
            self.schedule(0.0, true)?;
        }
        self.enqueue(data, callback)
    }
    fn enqueue(
        &self,
        data: WriteData,
        callback: Option<WriteCallback<S>>,
    ) -> Result<(), WriteError> {
        // Compound assignment reads the old pending value before a host getter.
        let pending = self.pending_units();
        let length = data.len()?;
        self.state.pending_units.set(pending + length as f64);
        self.state.data.borrow_mut().push_back(Rc::new(data));
        self.state.callbacks.borrow_mut().push_back(callback);
        Ok(())
    }
    /// Dispatch a fired zero-delay timer exactly once. The scheduler carries
    /// the supplied clock/result arguments in its request, not in global state.
    pub fn process(&self, last_time: f64, promise_result: bool) -> Result<(), WriteError> {
        self.state.timer.borrow_mut().take();
        self.inner_write(last_time, promise_result)
    }
    fn schedule(&self, last_time: f64, promise_result: bool) -> Result<(), WriteError> {
        if self.is_disposed() {
            return Ok(());
        }
        self.cancel_timer()?;
        let handle = self
            .state
            .scheduler
            .request(self.downgrade(), last_time, promise_result)?;
        *self.state.timer.borrow_mut() = Some(handle);
        Ok(())
    }
    fn cancel_timer(&self) -> Result<(), WriteError> {
        // Source keeps its token until clearTimeout returns. A host getter or
        // call may reenter the queue; never retain the slot borrow across it.
        let token = self.state.timer.borrow().as_ref().cloned();
        if let Some(token) = token {
            self.state.scheduler.cancel(token)?;
            self.state.timer.borrow_mut().take();
        }
        Ok(())
    }
    fn now(&self) -> f64 {
        self.state.scheduler.now()
    }
    fn clear_data(&self, offset: usize) {
        self.state.data.borrow_mut().clear();
        self.state.callbacks.borrow_mut().clear();
        self.state.pending_units.set(0.0);
        self.state.offset.set(offset);
    }
    fn fire_parsed(&self) -> Result<(), WriteError> {
        self.state.parsed.fire(&()).map_err(WriteError::Event)
    }
    /// Source-style explicit disposal propagates host cancellation failure,
    /// after clearing the remaining registered resources as DisposableStore does.
    pub fn try_dispose(&self) -> Result<(), WriteError> {
        if self.state.disposed.replace(true) {
            return Ok(());
        }
        let cancelled = self.cancel_timer();
        self.state.parsed.dispose();
        self.clear_data(0);
        cancelled
    }
    /// Infallible Rust cleanup reports failures through the host error channel.
    /// JavaScript-facing explicit dispose bindings must use try_dispose instead.
    pub fn dispose(&self) {
        if let Err(error) = self.try_dispose() {
            self.state.scheduler.report_error(error);
        }
    }
}

impl From<EmitError> for WriteError {
    fn from(error: EmitError) -> Self {
        Self::Event(error)
    }
}
