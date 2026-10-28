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
#[path = "write_buffer_types.rs"]
mod types;
pub use types::{WriteAction, WriteData, WriteError, WritePayload, WriteScheduler};

pub const DISCARD_WATERMARK: usize = 50_000_000;
pub const WRITE_TIMEOUT_MS: f64 = 12.0;
const LENGTH_THRESHOLD: usize = 50;
const SYNC_OFFSET: usize = 0x7fff_ffff;
type Action<S> =
    dyn Fn(&WriteBuffer<S>, &WriteData, Option<bool>) -> Result<WriteAction, WriteError>;
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
    scheduler: RefCell<S>,
    action: Rc<Action<S>>,
    parsed: Emitter<()>,
}
impl<S: WriteScheduler> Drop for State<S> {
    fn drop(&mut self) {
        if let Some(handle) = self.timer.get_mut().take() {
            self.scheduler.get_mut().cancel(handle);
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
                scheduler: RefCell::new(scheduler),
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
                self.enqueue(data, callback);
                return self.inner_write(0.0, true);
            }
            self.schedule(0.0, true)?;
        }
        self.enqueue(data, callback);
        Ok(())
    }
    fn enqueue(&self, data: WriteData, callback: Option<WriteCallback<S>>) {
        self.state
            .pending_units
            .set(self.pending_units() + data.len() as f64);
        self.state.data.borrow_mut().push_back(Rc::new(data));
        self.state.callbacks.borrow_mut().push_back(callback);
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
        let old = self.state.timer.borrow_mut().take();
        if let Some(handle) = old {
            self.state.scheduler.borrow_mut().cancel(handle);
        }
        let handle = self.state.scheduler.borrow_mut().request(
            self.downgrade(),
            last_time,
            promise_result,
        )?;
        *self.state.timer.borrow_mut() = Some(handle);
        Ok(())
    }
    fn now(&self) -> f64 {
        self.state.scheduler.borrow().now()
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
    pub fn dispose(&self) {
        if self.state.disposed.replace(true) {
            return;
        }
        let timer = self.state.timer.borrow_mut().take();
        if let Some(handle) = timer {
            self.state.scheduler.borrow_mut().cancel(handle);
        }
        self.state.parsed.dispose();
        self.clear_data(0);
    }
}

impl From<EmitError> for WriteError {
    fn from(error: EmitError) -> Self {
        Self::Event(error)
    }
}
