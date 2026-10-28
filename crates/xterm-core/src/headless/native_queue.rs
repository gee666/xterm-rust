//! Explicit native event pump for the portable WriteBuffer. No thread, sleep or
//! global executor is created. Wakes use a standard-library FIFO so readiness
//! order is retained and ready continuations precede the next timer callback.
use crate::input::write_buffer::{WeakWriteBuffer, WriteError, WriteScheduler};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, Sender, TryRecvError},
        Arc,
    },
    task::{Context, Wake, Waker},
    time::Instant,
};

struct Ready {
    id: u64,
    queue: Sender<u64>,
    queued: AtomicBool,
}
impl Ready {
    fn enqueue(&self) {
        if !self.queued.swap(true, Ordering::AcqRel) && self.queue.send(self.id).is_err() {
            // A retained waker may outlive the owning pump. With no receiver,
            // this is cancellation, not permission to execute work inline.
            self.queued.store(false, Ordering::Release);
        }
    }
}
impl Wake for Ready {
    fn wake(self: Arc<Self>) {
        self.enqueue();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.enqueue();
    }
}
struct LocalFuture {
    future: Pin<Box<dyn Future<Output = ()>>>,
    ready: Arc<Ready>,
}
enum Timer {
    Write {
        queue: WeakWriteBuffer<NativeWriteScheduler>,
        last_time: f64,
        result: bool,
    },
    Callback(Box<dyn FnOnce() -> Result<(), WriteError>>),
}
struct State {
    clock: Rc<dyn Fn() -> f64>,
    next_id: u64,
    next_future_id: u64,
    timers: BTreeMap<u64, Timer>,
    futures: BTreeMap<u64, LocalFuture>,
    ready_tx: Sender<u64>,
    ready_rx: Receiver<u64>,
    errors: Vec<WriteError>,
    closed: bool,
}

/// Cloneable scheduler handle. The NativeWritePump owns execution and lifetime.
#[derive(Clone)]
pub struct NativeWriteScheduler {
    state: Rc<RefCell<State>>,
}
impl NativeWriteScheduler {
    fn schedule(&self, timer: Timer) -> Result<u64, WriteError> {
        let mut state = self.state.borrow_mut();
        if state.closed {
            return Err(WriteError::Schedule("native write pump is closed".into()));
        }
        let id = state.next_id;
        state.next_id = id
            .checked_add(1)
            .ok_or_else(|| WriteError::Schedule("native timer identifier exhausted".into()))?;
        state.timers.insert(id, timer);
        Ok(id)
    }
    /// Portable native task adapters share write timers' FIFO and error channel.
    pub fn request_callback(
        &self,
        callback: impl FnOnce() -> Result<(), WriteError> + 'static,
    ) -> Result<u64, WriteError> {
        self.schedule(Timer::Callback(Box::new(callback)))
    }
    /// Native timer removal cannot fail. Drop captures outside the state borrow
    /// because their owners may cancel or request another timer during Drop.
    pub fn cancel_timer(&self, handle: u64) {
        let removed = self.state.borrow_mut().timers.remove(&handle);
        drop(removed);
    }
}
impl WriteScheduler for NativeWriteScheduler {
    type Handle = u64;
    fn request(
        &self,
        queue: WeakWriteBuffer<Self>,
        last_time: f64,
        promise_result: bool,
    ) -> Result<Self::Handle, WriteError> {
        self.schedule(Timer::Write {
            queue,
            last_time,
            result: promise_result,
        })
    }
    fn cancel(&self, handle: Self::Handle) -> Result<(), WriteError> {
        self.cancel_timer(handle);
        Ok(())
    }
    fn now(&self) -> f64 {
        let clock = Rc::clone(&self.state.borrow().clock);
        clock()
    }
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()>>>) {
        let mut state = self.state.borrow_mut();
        if state.closed {
            state.errors.push(WriteError::Schedule(
                "cannot spawn on a closed native write pump".into(),
            ));
            return;
        }
        let id = state.next_future_id;
        let Some(next) = id.checked_add(1) else {
            state.errors.push(WriteError::Schedule(
                "native future identifier exhausted".into(),
            ));
            return;
        };
        state.next_future_id = next;
        let ready = Arc::new(Ready {
            id,
            queue: state.ready_tx.clone(),
            queued: AtomicBool::new(false),
        });
        ready.enqueue();
        state.futures.insert(id, LocalFuture { future, ready });
    }
    fn report_error(&self, error: WriteError) {
        self.state.borrow_mut().errors.push(error);
    }
}

/// A poll snapshots ready timers, drains ready continuations before and between
/// those timers, and leaves newly requested timers for the next host poll.
/// Futures are polled only after a wake, never retried merely for being Pending.
/// Like JavaScript microtasks, a user continuation that continually wakes itself
/// can starve timers. The pump does not invent a sleep or silently discard it.
/// Dropping the pump cancels owned timers/futures and breaks pending cycles.
pub struct NativeWritePump {
    state: Rc<RefCell<State>>,
}
impl Default for NativeWritePump {
    fn default() -> Self {
        Self::new()
    }
}
impl NativeWritePump {
    pub fn new() -> Self {
        let origin = Instant::now();
        Self::with_clock(move || origin.elapsed().as_secs_f64() * 1000.0)
    }
    /// Host clock in milliseconds. Controlled source comparisons must inject the
    /// same clock rather than relying on OS preemption around the 12ms budget.
    pub fn with_clock(clock: impl Fn() -> f64 + 'static) -> Self {
        let (ready_tx, ready_rx) = mpsc::channel();
        Self {
            state: Rc::new(RefCell::new(State {
                clock: Rc::new(clock),
                next_id: 0,
                next_future_id: 0,
                timers: BTreeMap::new(),
                futures: BTreeMap::new(),
                ready_tx,
                ready_rx,
                errors: Vec::new(),
                closed: false,
            })),
        }
    }
    pub fn scheduler(&self) -> NativeWriteScheduler {
        NativeWriteScheduler {
            state: Rc::clone(&self.state),
        }
    }
    pub fn pending_tasks(&self) -> usize {
        let state = self.state.borrow();
        state.timers.len() + state.futures.len()
    }
    pub fn poll(&self) -> Result<usize, Vec<WriteError>> {
        self.check_errors()?;
        let timers: Vec<_> = self.state.borrow().timers.keys().copied().collect();
        let mut executed = self.poll_ready_futures();
        self.check_errors()?;
        for id in timers {
            // Live storage lets an earlier callback cancel a later timer from
            // this same batch. Never hold its borrow across dispatch.
            let timer = self.state.borrow_mut().timers.remove(&id);
            if let Some(timer) = timer {
                let result = match timer {
                    Timer::Write {
                        queue,
                        last_time,
                        result,
                    } => queue.upgrade().map(|queue| {
                        executed += 1;
                        queue.process(last_time, result)
                    }),
                    Timer::Callback(callback) => {
                        executed += 1;
                        Some(callback())
                    }
                };
                if let Some(Err(error)) = result {
                    self.state.borrow_mut().errors.push(error);
                }
            }
            self.check_errors()?;
            executed += self.poll_ready_futures();
            self.check_errors()?;
        }
        Ok(executed)
    }
    fn check_errors(&self) -> Result<(), Vec<WriteError>> {
        let errors = std::mem::take(&mut self.state.borrow_mut().errors);
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    fn poll_ready_futures(&self) -> usize {
        let mut executed = 0;
        loop {
            let ready = self.state.borrow().ready_rx.try_recv();
            let id = match ready {
                Ok(id) => id,
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    self.state.borrow_mut().errors.push(WriteError::Schedule(
                        "native wake queue disconnected".into(),
                    ));
                    break;
                }
            };
            let task = self.state.borrow_mut().futures.remove(&id);
            // A future may complete while an already-queued wake remains.
            let Some(mut task) = task else {
                continue;
            };
            task.ready.queued.store(false, Ordering::Release);
            let waker = Waker::from(Arc::clone(&task.ready));
            let mut context = Context::from_waker(&waker);
            executed += 1;
            if task.future.as_mut().poll(&mut context).is_pending() {
                self.state.borrow_mut().futures.insert(id, task);
            }
            // Surface the uncaught error before running its separately queued
            // false continuation. Remaining work stays owned for the next poll.
            if !self.state.borrow().errors.is_empty() {
                break;
            }
        }
        executed
    }
}
impl Drop for NativeWritePump {
    fn drop(&mut self) {
        // Captured queue owners may cancel timers during destruction. Release
        // futures only after the state borrow ends to permit that reentrancy.
        let (timers, futures) = {
            let mut state = self.state.borrow_mut();
            state.closed = true;
            (
                std::mem::take(&mut state.timers),
                std::mem::take(&mut state.futures),
            )
        };
        drop(timers);
        drop(futures);
    }
}
