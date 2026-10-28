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

#[derive(Debug)]
pub enum CleanupError<E> {
    Buffer(BufferError),
    Schedule(E),
    ScheduledCallback(Rc<E>),
}
impl<E: fmt::Display> fmt::Display for CleanupError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Buffer(error) => error.fmt(f),
            Self::Schedule(error) => error.fmt(f),
            Self::ScheduledCallback(error) => error.fmt(f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for CleanupError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Buffer(error) => Some(error),
            Self::Schedule(error) => Some(error),
            Self::ScheduledCallback(error) => Some(error.as_ref()),
        }
    }
}
impl<E> From<BufferError> for CleanupError<E> {
    fn from(error: BufferError) -> Self {
        Self::Buffer(error)
    }
}
type Failure<E> = Rc<RefCell<Option<CleanupError<E>>>>;

/// One host scheduling owner for a shared buffer. The host dispatches the queue
/// when its requested idle/priority callback fires. No clock or sleep is invented
/// by the core. Dropping this adapter unregisters and cancels outstanding work.
///
/// A scheduling failure during resize propagates as BufferError::Dispatch; call
/// take_error to recover the underlying typed host error. Work remains pending
/// until the host retries or another resize replaces it, as in the source queue.
pub struct BufferCleanup<S: Scheduler> {
    queue: TaskQueue<S>,
    buffer: Weak<RefCell<Buffer>>,
    failure: Failure<S::Error>,
    subscription: DisposableHandle,
    attached: Rc<Cell<bool>>,
}
impl<S: Scheduler + 'static> BufferCleanup<S>
where
    S::Error: std::error::Error + 'static,
{
    pub fn new(
        buffer: &Rc<RefCell<Buffer>>,
        scheduler: S,
        supports_idle: bool,
    ) -> Result<Self, CleanupError<S::Error>> {
        Self::with_queue(buffer, TaskQueue::idle(scheduler, supports_idle))
    }
    /// Attach an empty dedicated queue already configured with the host's weak
    /// callback dispatcher. The host may retain its queue handle for dispatch.
    pub fn with_queue(
        buffer: &Rc<RefCell<Buffer>>,
        queue: TaskQueue<S>,
    ) -> Result<Self, CleanupError<S::Error>> {
        if queue.remaining_tasks() != 0 || queue.has_pending_callback() {
            return Err(BufferError::CleanupQueueBusy.into());
        }
        let state = buffer.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
        if state.cleanup_attached.replace(true) {
            return Err(BufferError::CleanupAlreadyAttached.into());
        }
        let failure = Rc::new(RefCell::new(None));
        let weak = Rc::downgrade(buffer);
        let queue_ref = queue.clone();
        let failure_ref = Rc::clone(&failure);
        let weak_ref = weak.clone();
        let subscription =
            state.cleanup_events.event().try_subscribe(move |enabled| {
                match schedule(
                    &queue_ref,
                    *enabled,
                    weak_ref.clone(),
                    Rc::clone(&failure_ref),
                ) {
                    Ok(()) => Ok(()),
                    Err(error) => {
                        let error = Rc::new(error);
                        *failure_ref.borrow_mut() =
                            Some(CleanupError::ScheduledCallback(Rc::clone(&error)));
                        Err(EmitError::Callback(error))
                    }
                }
            });
        let adapter = Self {
            queue,
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
        self.queue.has_pending_callback()
    }
    /// Dispatch one real idle callback and report borrow, storage or host failure.
    pub fn process_idle(
        &self,
        time_remaining: impl FnMut() -> f64,
    ) -> Result<(), CleanupError<S::Error>> {
        self.queue
            .process(time_remaining)
            .map_err(CleanupError::Schedule)?;
        self.take_error().map_or(Ok(()), Err)
    }
    pub fn process_priority(&self) -> Result<(), CleanupError<S::Error>> {
        self.queue
            .process_priority()
            .map_err(CleanupError::Schedule)?;
        self.take_error().map_or(Ok(()), Err)
    }
    /// Explicit recovery after handling a temporary borrow or host scheduling error.
    pub fn retry_pending(&self) -> Result<(), CleanupError<S::Error>> {
        let pending = if let Some(buffer) = self.buffer.upgrade() {
            let state = buffer.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
            state.has_pending_cleanup()
        } else {
            false
        };
        schedule(
            &self.queue,
            pending,
            self.buffer.clone(),
            Rc::clone(&self.failure),
        )
        .map_err(CleanupError::Schedule)
    }
    pub fn take_error(&self) -> Option<CleanupError<S::Error>> {
        self.failure.borrow_mut().take()
    }
}
impl<S: Scheduler> Drop for BufferCleanup<S> {
    fn drop(&mut self) {
        self.subscription.dispose();
        self.queue.clear();
        self.attached.set(false);
    }
}
fn schedule<S: Scheduler + 'static>(
    queue: &TaskQueue<S>,
    enabled: bool,
    buffer: Weak<RefCell<Buffer>>,
    failure: Failure<S::Error>,
) -> Result<(), S::Error> {
    queue.clear();
    if enabled {
        queue.enqueue(move |_| {
            let Some(buffer) = buffer.upgrade() else {
                return false;
            };
            let result = buffer
                .try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)
                .and_then(|mut state| state.poll_cleanup());
            match result {
                Ok(pending) => pending,
                Err(error) => {
                    *failure.borrow_mut() = Some(CleanupError::Buffer(error));
                    false
                }
            }
        })?;
    }
    Ok(())
}
