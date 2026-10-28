// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Connect Buffer's deferred memory cleanup to the existing host task queue.
use super::{Buffer, BufferError};
use crate::{
    event::EmitError,
    lifecycle::DisposableHandle,
    task_queue::{Scheduler, TaskQueue},
};
use std::{
    cell::{Cell, RefCell},
    fmt,
    rc::{Rc, Weak},
};
#[path = "cleanup_driver.rs"]
mod backend;
use backend::Driver;

#[derive(Debug)]
pub enum CleanupError<E> {
    Buffer(BufferError),
    /// Failure from a scheduler or the caller-supplied queue's error domain.
    Schedule(E),
    ScheduledCallback(Rc<E>),
}
impl<E: fmt::Display> fmt::Display for CleanupError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Buffer(e) => e.fmt(f),
            Self::Schedule(e) => e.fmt(f),
            Self::ScheduledCallback(e) => e.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for CleanupError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Buffer(e) => Some(e),
            Self::Schedule(e) => Some(e),
            Self::ScheduledCallback(e) => Some(e.as_ref()),
        }
    }
}
impl<E> From<BufferError> for CleanupError<E> {
    fn from(error: BufferError) -> Self {
        Self::Buffer(error)
    }
}
type Failure<E> = Rc<RefCell<Option<CleanupError<E>>>>;

/// One host scheduling owner per buffer. Tasks hold only a weak buffer reference.
/// Dispatch failures leave the current queue task and index intact, just as an
/// upstream throw does. Dropping this adapter cancels its outstanding work.
pub struct BufferCleanup<S: Scheduler> {
    driver: Rc<Driver<S>>,
    buffer: Weak<RefCell<Buffer>>,
    failure: Failure<S::Error>,
    subscription: DisposableHandle,
    attached: Rc<Cell<bool>>,
}
impl<S: Scheduler + 'static> BufferCleanup<S>
where
    S::Error: 'static,
{
    pub fn new(
        buffer: &Rc<RefCell<Buffer>>,
        scheduler: S,
        idle: bool,
    ) -> Result<Self, CleanupError<S::Error>>
    where
        S::Error: std::error::Error,
    {
        Self::attach(buffer, Driver::owned(scheduler, idle), |error| {
            EmitError::Callback(error)
        })
    }
    /// Inject a dedicated queue whose host error type can retain Buffer errors.
    pub fn with_queue(
        buffer: &Rc<RefCell<Buffer>>,
        queue: TaskQueue<S>,
    ) -> Result<Self, CleanupError<S::Error>>
    where
        S::Error: std::error::Error + From<BufferError>,
    {
        Self::with_queue_map(buffer, queue, S::Error::from, |error| {
            EmitError::Callback(error)
        })
    }
    /// Configure an opaque host error domain, for example a browser JsValue.
    /// Mappers must preserve the original thrown object rather than stringify it.
    /// Callback errors return immediately through the supplied queue dispatcher;
    /// callers need not poll take_error after an otherwise successful dispatch.
    pub fn with_queue_map(
        buffer: &Rc<RefCell<Buffer>>,
        queue: TaskQueue<S>,
        convert: impl Fn(BufferError) -> S::Error + 'static,
        wrap_scheduler: impl Fn(Rc<S::Error>) -> EmitError + 'static,
    ) -> Result<Self, CleanupError<S::Error>> {
        Self::attach(buffer, Driver::injected(queue, convert), wrap_scheduler)
    }
    fn attach(
        buffer: &Rc<RefCell<Buffer>>,
        driver: Driver<S>,
        wrap_scheduler: impl Fn(Rc<S::Error>) -> EmitError + 'static,
    ) -> Result<Self, CleanupError<S::Error>> {
        if !driver.empty() {
            return Err(BufferError::CleanupQueueBusy.into());
        }
        let state = buffer.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
        if state.cleanup_attached.replace(true) {
            return Err(BufferError::CleanupAlreadyAttached.into());
        }
        let driver = Rc::new(driver);
        let failure = Rc::new(RefCell::new(None));
        let weak = Rc::downgrade(buffer);
        let callback_driver = Rc::clone(&driver);
        let callback_buffer = weak.clone();
        let callback_failure = Rc::clone(&failure);
        let subscription = state.cleanup_events.event().try_subscribe(move |enabled| {
            match schedule(&callback_driver, *enabled, callback_buffer.clone()) {
                Ok(()) => Ok(()),
                Err(CleanupError::Schedule(error)) => {
                    let error = Rc::new(error);
                    *callback_failure.borrow_mut() =
                        Some(CleanupError::ScheduledCallback(Rc::clone(&error)));
                    Err(wrap_scheduler(error))
                }
                Err(CleanupError::ScheduledCallback(error)) => {
                    *callback_failure.borrow_mut() =
                        Some(CleanupError::ScheduledCallback(Rc::clone(&error)));
                    Err(wrap_scheduler(error))
                }
                Err(CleanupError::Buffer(error)) => {
                    // Native diagnostic values may be copied; any nested actual
                    // callback object remains shared through EmitError's Rc.
                    *callback_failure.borrow_mut() = Some(CleanupError::Buffer(error.clone()));
                    Err(EmitError::callback(error))
                }
            }
        });
        let adapter = Self {
            driver,
            buffer: weak,
            failure,
            subscription,
            attached: Rc::clone(&state.cleanup_attached),
        };
        drop(state);
        adapter.retry_pending()?;
        Ok(adapter)
    }
    pub fn has_pending_callback(&self) -> bool {
        self.driver.pending()
    }
    pub fn process_idle(
        &self,
        remaining: impl FnMut() -> f64,
    ) -> Result<(), CleanupError<S::Error>> {
        self.driver.process(remaining)
    }
    pub fn process_priority(&self) -> Result<(), CleanupError<S::Error>> {
        self.driver.priority()
    }
    pub fn retry_pending(&self) -> Result<(), CleanupError<S::Error>> {
        let pending = if let Some(buffer) = self.buffer.upgrade() {
            let state = buffer.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
            state.has_pending_cleanup()
        } else {
            false
        };
        schedule(&self.driver, pending, self.buffer.clone())
    }
    /// Retrieve a resize-time scheduling failure for diagnostics. It was already
    /// propagated through BufferError::Dispatch and is never rethrown on success.
    pub fn take_error(&self) -> Option<CleanupError<S::Error>> {
        self.failure.borrow_mut().take()
    }
}
impl<S: Scheduler> Drop for BufferCleanup<S> {
    fn drop(&mut self) {
        self.subscription.dispose();
        self.driver.clear();
        self.attached.set(false);
    }
}
fn schedule<S: Scheduler + 'static>(
    driver: &Driver<S>,
    enabled: bool,
    buffer: Weak<RefCell<Buffer>>,
) -> Result<(), CleanupError<S::Error>>
where
    S::Error: 'static,
{
    driver.clear();
    if enabled {
        driver.enqueue(Rc::new(move || {
            let Some(buffer) = buffer.upgrade() else {
                return Ok(false);
            };
            let mut state = buffer
                .try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?;
            state.poll_cleanup()
        }))?;
    }
    Ok(())
}
