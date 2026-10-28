// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{BufferError, CleanupError};
use crate::task_queue::{CallbackKind, Scheduler, TaskQueue};
use std::rc::Rc;

type Work = Rc<dyn Fn() -> Result<bool, BufferError>>;
/// A standalone scheduler can expose native Buffer errors without forcing its
/// host error type to invent a representation for them. Injected queues instead
/// use the host's explicit mapping so their existing dispatcher sees failures.
pub(super) enum Driver<S: Scheduler> {
    Owned(TaskQueue<WrappedScheduler<S>>),
    Injected {
        queue: TaskQueue<S>,
        convert: Rc<dyn Fn(BufferError) -> S::Error>,
    },
}
pub(super) struct WrappedScheduler<S>(S);
impl<S: Scheduler> Scheduler for WrappedScheduler<S> {
    type Handle = S::Handle;
    type Error = CleanupError<S::Error>;
    fn request(&self, kind: CallbackKind) -> Result<Self::Handle, Self::Error> {
        self.0.request(kind).map_err(CleanupError::Schedule)
    }
    fn cancel(&self, handle: Self::Handle) {
        self.0.cancel(handle);
    }
    fn callback_fired(&self, handle: Self::Handle) {
        self.0.callback_fired(handle);
    }
    fn now(&self) -> f64 {
        self.0.now()
    }
    fn deadline_exceeded(&self, ms: f64) -> Result<(), Self::Error> {
        self.0.deadline_exceeded(ms).map_err(CleanupError::Schedule)
    }
}
impl<S: Scheduler> Driver<S> {
    pub(super) fn clear(&self) {
        match self {
            Self::Owned(q) => q.clear(),
            Self::Injected { queue, .. } => queue.clear(),
        }
    }
    pub(super) fn pending(&self) -> bool {
        match self {
            Self::Owned(q) => q.has_pending_callback(),
            Self::Injected { queue, .. } => queue.has_pending_callback(),
        }
    }
    pub(super) fn empty(&self) -> bool {
        match self {
            Self::Owned(q) => q.remaining_tasks() == 0 && !q.has_pending_callback(),
            Self::Injected { queue, .. } => {
                queue.remaining_tasks() == 0 && !queue.has_pending_callback()
            }
        }
    }
    pub(super) fn process(
        &self,
        remaining: impl FnMut() -> f64,
    ) -> Result<(), CleanupError<S::Error>> {
        match self {
            Self::Owned(q) => q.process(remaining),
            Self::Injected { queue, .. } => {
                queue.process(remaining).map_err(CleanupError::Schedule)
            }
        }
    }
    pub(super) fn priority(&self) -> Result<(), CleanupError<S::Error>> {
        match self {
            Self::Owned(q) => q.process_priority(),
            Self::Injected { queue, .. } => {
                queue.process_priority().map_err(CleanupError::Schedule)
            }
        }
    }
}
impl<S: Scheduler + 'static> Driver<S>
where
    S::Error: 'static,
{
    pub(super) fn owned(scheduler: S, idle: bool) -> Self {
        Self::Owned(TaskQueue::idle(WrappedScheduler(scheduler), idle))
    }
    pub(super) fn injected(
        queue: TaskQueue<S>,
        convert: impl Fn(BufferError) -> S::Error + 'static,
    ) -> Self {
        Self::Injected {
            queue,
            convert: Rc::new(convert),
        }
    }
    pub(super) fn enqueue(&self, work: Work) -> Result<(), CleanupError<S::Error>> {
        match self {
            Self::Owned(q) => q.enqueue_try(move |_| work().map_err(CleanupError::Buffer)),
            Self::Injected { queue, convert } => {
                let convert = Rc::clone(convert);
                queue
                    .enqueue_try(move |_| work().map_err(convert.as_ref()))
                    .map_err(CleanupError::Schedule)
            }
        }
    }
}
