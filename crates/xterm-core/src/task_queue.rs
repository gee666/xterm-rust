// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
//! Cooperative FIFO work with host-controlled callback dispatch and clocks.
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

pub const PRIORITY_BUDGET_MS: f64 = 16.0;
const DEADLINE_HEADROOM: f64 = 1.5;
const OVERRUN_WARNING_MS: f64 = 20.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CallbackKind {
    Priority,
    Idle,
}

/// The adapter schedules one future call to `TaskQueue::process`. Requests must
/// not invoke it inline. Idle callbacks supply the host's remaining-time value;
/// priority callbacks call `process_priority` instead. Handles are never zero
/// sentinels. Cancellation must prevent future dispatch of that handle.
pub trait Scheduler {
    type Handle;
    type Error;
    fn request(&self, kind: CallbackKind) -> Result<Self::Handle, Self::Error>;
    fn cancel(&self, handle: Self::Handle);
    /// Releases a fired callback's resources without canceling a host callback
    /// which has already run. RAII handles may override this to disarm cleanup.
    fn callback_fired(&self, handle: Self::Handle) {
        drop(handle);
    }
    /// Milliseconds, matching performance.now().
    fn now(&self) -> f64;
    fn deadline_exceeded(&self, milliseconds: f64) -> Result<(), Self::Error>;
}

type Task<S> = Rc<dyn Fn(&TaskQueue<S>) -> bool>;
struct State<S: Scheduler> {
    tasks: RefCell<Vec<Task<S>>>,
    index: Cell<usize>,
    pending: RefCell<Option<S::Handle>>,
    scheduler: S,
    kind: CallbackKind,
}
impl<S: Scheduler> Drop for State<S> {
    fn drop(&mut self) {
        if let Some(handle) = self.pending.get_mut().take() {
            self.scheduler.cancel(handle);
        }
    }
}

/// Cloneable queue handle. Tasks return true to repeat and false to advance.
/// Callbacks receive the queue so enqueue/clear/flush can reenter without a
/// borrow held by the dispatcher. Use Cell/RefCell for mutable callback state.
/// Do not capture a strong queue handle in a stored task; use its argument to
/// avoid an Rc cycle. Dropping the last handle cancels outstanding host work.
pub struct TaskQueue<S: Scheduler> {
    state: Rc<State<S>>,
}
/// Non-owning dispatch handle for host callbacks. A callback cannot keep its
/// queue alive after the last public owner has dropped it.
pub struct WeakTaskQueue<S: Scheduler> {
    state: Weak<State<S>>,
}
impl<S: Scheduler> Clone for WeakTaskQueue<S> {
    fn clone(&self) -> Self {
        Self {
            state: self.state.clone(),
        }
    }
}
impl<S: Scheduler> WeakTaskQueue<S> {
    pub fn upgrade(&self) -> Option<TaskQueue<S>> {
        self.state.upgrade().map(|state| TaskQueue { state })
    }
}
impl<S: Scheduler> Clone for TaskQueue<S> {
    fn clone(&self) -> Self {
        Self {
            state: Rc::clone(&self.state),
        }
    }
}
impl<S: Scheduler> TaskQueue<S> {
    pub fn new(scheduler: S, kind: CallbackKind) -> Self {
        Self {
            state: Rc::new(State {
                tasks: RefCell::new(Vec::new()),
                index: Cell::new(0),
                pending: RefCell::new(None),
                scheduler,
                kind,
            }),
        }
    }
    /// Implements upstream IdleTaskQueue's priority fallback without detecting
    /// browser globals in the portable core.
    pub fn idle(scheduler: S, supports_idle: bool) -> Self {
        Self::new(
            scheduler,
            if supports_idle {
                CallbackKind::Idle
            } else {
                CallbackKind::Priority
            },
        )
    }
    pub fn downgrade(&self) -> WeakTaskQueue<S> {
        WeakTaskQueue {
            state: Rc::downgrade(&self.state),
        }
    }
    /// Failed scheduling leaves the appended task queued with no pending token,
    /// as an exception from the upstream request API does. Callers may retry by
    /// enqueueing more work, flush synchronously or clear the queue.
    pub fn enqueue(&self, task: impl Fn(&Self) -> bool + 'static) -> Result<(), S::Error> {
        self.state.tasks.borrow_mut().push(Rc::new(task));
        self.start()
    }
    pub fn flush(&self) {
        while let Some(task) = self.current_task() {
            if !task(self) {
                self.state.index.set(self.state.index.get() + 1);
            }
        }
        self.clear();
    }
    pub fn clear(&self) {
        let handle = self.state.pending.borrow_mut().take();
        if let Some(handle) = handle {
            self.state.scheduler.cancel(handle);
        }
        self.state.index.set(0);
        self.state.tasks.borrow_mut().clear();
    }
    pub fn has_pending_callback(&self) -> bool {
        self.state.pending.borrow().is_some()
    }
    pub fn remaining_tasks(&self) -> usize {
        self.state
            .tasks
            .borrow()
            .len()
            .saturating_sub(self.state.index.get())
    }
    /// Dispatch a fired host callback exactly once. A deadline is deliberately
    /// queried after every task, even after the last task, as upstream does.
    pub fn process(&self, mut time_remaining: impl FnMut() -> f64) -> Result<(), S::Error> {
        let handle = self.state.pending.borrow_mut().take();
        if let Some(handle) = handle {
            self.state.scheduler.callback_fired(handle);
        }
        let mut longest = 0.0_f64;
        let mut previous_remaining = time_remaining();
        while let Some(task) = self.current_task() {
            let start = self.now();
            if !task(self) {
                self.state.index.set(self.state.index.get() + 1);
            }
            let duration = js_max(1.0, self.now() - start);
            longest = js_max(longest, duration);
            let remaining = time_remaining();
            if longest * DEADLINE_HEADROOM > remaining {
                let overrun = previous_remaining - duration;
                if overrun < -OVERRUN_WARNING_MS {
                    // JS Math.round ties towards +infinity, unlike f64::round.
                    self.state
                        .scheduler
                        .deadline_exceeded((overrun + 0.5).floor().abs())?;
                }
                return self.start();
            }
            previous_remaining = remaining;
        }
        self.clear();
        Ok(())
    }
    pub fn process_priority(&self) -> Result<(), S::Error> {
        let end = self.now() + PRIORITY_BUDGET_MS;
        self.process(|| js_max(0.0, end - self.now()))
    }
    fn now(&self) -> f64 {
        self.state.scheduler.now()
    }
    fn current_task(&self) -> Option<Task<S>> {
        self.state
            .tasks
            .borrow()
            .get(self.state.index.get())
            .cloned()
    }
    fn start(&self) -> Result<(), S::Error> {
        if self.has_pending_callback() {
            return Ok(());
        }
        let handle = self.state.scheduler.request(self.state.kind)?;
        *self.state.pending.borrow_mut() = Some(handle);
        Ok(())
    }
}
fn js_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}

/// One replaceable idle task. Disposal clears queued work; a later set is
/// allowed, matching upstream rather than introducing a disposed-state gate.
pub struct DebouncedIdleTask<S: Scheduler> {
    queue: TaskQueue<S>,
}
impl<S: Scheduler> DebouncedIdleTask<S> {
    pub fn new(scheduler: S, supports_idle: bool) -> Self {
        Self {
            queue: TaskQueue::idle(scheduler, supports_idle),
        }
    }
    pub fn set(&self, task: impl Fn(&TaskQueue<S>) -> bool + 'static) -> Result<(), S::Error> {
        self.queue.clear();
        self.queue.enqueue(task)
    }
    pub fn flush(&self) {
        self.queue.flush();
    }
    pub fn dispose(&self) {
        self.queue.clear();
    }
    pub fn process(&self, time_remaining: impl FnMut() -> f64) -> Result<(), S::Error> {
        self.queue.process(time_remaining)
    }
    pub fn process_priority(&self) -> Result<(), S::Error> {
        self.queue.process_priority()
    }
}
