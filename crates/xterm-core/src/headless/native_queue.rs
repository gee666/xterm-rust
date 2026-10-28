//! Explicit native event pump for the portable WriteBuffer. No thread, sleep or
//! global executor is created. The owner polls ready work and handles each error.
use crate::input::write_buffer::{WeakWriteBuffer, WriteError, WriteScheduler};
use std::{
    cell::RefCell,
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    task::{Context, Poll, Wake, Waker},
    time::Instant,
};

struct Ready(AtomicBool);
impl Wake for Ready {
    fn wake(self: Arc<Self>) {
        self.0.store(true, Ordering::Release);
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.0.store(true, Ordering::Release);
    }
}
struct LocalFuture {
    future: Pin<Box<dyn Future<Output = ()>>>,
    ready: Arc<Ready>,
}
struct Timer {
    queue: WeakWriteBuffer<NativeWriteScheduler>,
    last_time: f64,
    result: bool,
}
struct State {
    origin: Instant,
    next_id: u64,
    timers: BTreeMap<u64, Timer>,
    futures: Vec<LocalFuture>,
    errors: Vec<WriteError>,
    closed: bool,
}

/// Cloneable scheduler handle. The NativeWritePump owns execution and lifetime.
#[derive(Clone)]
pub struct NativeWriteScheduler {
    state: Rc<RefCell<State>>,
}
impl WriteScheduler for NativeWriteScheduler {
    type Handle = u64;
    fn request(
        &mut self,
        queue: WeakWriteBuffer<Self>,
        last_time: f64,
        promise_result: bool,
    ) -> Result<Self::Handle, WriteError> {
        let mut state = self.state.borrow_mut();
        if state.closed {
            return Err(WriteError::Schedule("native write pump is closed".into()));
        }
        let id = state.next_id;
        state.next_id = id
            .checked_add(1)
            .ok_or_else(|| WriteError::Schedule("native timer identifier exhausted".into()))?;
        state.timers.insert(
            id,
            Timer {
                queue,
                last_time,
                result: promise_result,
            },
        );
        Ok(id)
    }
    fn cancel(&mut self, handle: Self::Handle) {
        self.state.borrow_mut().timers.remove(&handle);
    }
    fn now(&self) -> f64 {
        self.state.borrow().origin.elapsed().as_secs_f64() * 1000.0
    }
    fn spawn(&mut self, future: Pin<Box<dyn Future<Output = ()>>>) {
        let mut state = self.state.borrow_mut();
        if state.closed {
            state.errors.push(WriteError::Schedule(
                "cannot spawn on a closed native write pump".into(),
            ));
            return;
        }
        state.futures.push(LocalFuture {
            future,
            ready: Arc::new(Ready(AtomicBool::new(true))),
        });
    }
    fn report_error(&mut self, error: WriteError) {
        self.state.borrow_mut().errors.push(error);
    }
}

/// A single poll executes only the timers ready at its start, then polls ready
/// local futures once. Newly requested timers wait for the next host poll.
/// Future wakers only mark readiness; they never reenter the terminal inline.
/// Dropping the pump cancels its owned timers/futures and breaks pending cycles.
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
        Self {
            state: Rc::new(RefCell::new(State {
                origin: Instant::now(),
                next_id: 0,
                timers: BTreeMap::new(),
                futures: Vec::new(),
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
        let timers: Vec<_> = self.state.borrow().timers.keys().copied().collect();
        let mut executed = 0;
        for id in timers {
            // Keep not-yet-fired timers in live storage so an earlier callback
            // can cancel a later timer from this same poll batch.
            let Some(timer) = self.state.borrow_mut().timers.remove(&id) else {
                continue;
            };
            if let Some(queue) = timer.queue.upgrade() {
                executed += 1;
                if let Err(error) = queue.process(timer.last_time, timer.result) {
                    self.state.borrow_mut().errors.push(error);
                }
            }
        }
        let futures = std::mem::take(&mut self.state.borrow_mut().futures);
        let mut pending = Vec::new();
        for mut task in futures {
            if task.ready.0.swap(false, Ordering::AcqRel) {
                executed += 1;
                let waker = Waker::from(Arc::clone(&task.ready));
                let mut context = Context::from_waker(&waker);
                if let Poll::Ready(()) = task.future.as_mut().poll(&mut context) {
                    continue;
                }
            }
            pending.push(task);
        }
        self.state.borrow_mut().futures.extend(pending);
        let errors = std::mem::take(&mut self.state.borrow_mut().errors);
        if errors.is_empty() {
            Ok(executed)
        } else {
            Err(errors)
        }
    }
}
impl Drop for NativeWritePump {
    fn drop(&mut self) {
        // Drop futures outside a RefCell borrow: captured queue owners may cancel
        // their timers during destruction and therefore borrow this same state.
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
