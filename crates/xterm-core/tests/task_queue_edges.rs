use std::{
    cell::{Cell, RefCell},
    panic::{catch_unwind, AssertUnwindSafe},
    rc::Rc,
};
use xterm_core::task_queue::{CallbackKind, Scheduler, TaskQueue};

#[derive(Clone, Default)]
struct Host {
    clock: Rc<Cell<f64>>,
    requested: Rc<Cell<usize>>,
    cancelled: Rc<RefCell<Vec<usize>>>,
}
impl Scheduler for Host {
    type Handle = usize;
    type Error = std::convert::Infallible;
    fn request(&self, _: CallbackKind) -> Result<usize, Self::Error> {
        let id = self.requested.get() + 1;
        self.requested.set(id);
        Ok(id)
    }
    fn cancel(&self, id: usize) {
        self.cancelled.borrow_mut().push(id);
    }
    fn now(&self) -> f64 {
        self.clock.get()
    }
    fn deadline_exceeded(&self, _: f64) -> Result<(), Self::Error> {
        panic!("unexpected warning");
    }
}
#[derive(Clone)]
struct FallibleHost {
    host: Host,
    fail: Rc<Cell<bool>>,
}
impl Scheduler for FallibleHost {
    type Handle = usize;
    type Error = &'static str;
    fn request(&self, kind: CallbackKind) -> Result<usize, Self::Error> {
        if self.fail.get() {
            Err("host request failed")
        } else {
            Ok(self.host.request(kind).unwrap())
        }
    }
    fn cancel(&self, id: usize) {
        self.host.cancel(id);
    }
    fn now(&self) -> f64 {
        self.host.now()
    }
    fn deadline_exceeded(&self, ms: f64) -> Result<(), Self::Error> {
        self.host
            .deadline_exceeded(ms)
            .map_err(|never| match never {})
    }
}
#[test]
fn request_failure_releases_borrows_and_preserves_queue_state() {
    let fail = Rc::new(Cell::new(true));
    let host = Host::default();
    let queue = TaskQueue::idle(
        FallibleHost {
            host,
            fail: fail.clone(),
        },
        true,
    );
    assert_eq!(queue.enqueue(|_| false), Err("host request failed"));
    assert_eq!(queue.remaining_tasks(), 1);
    assert!(!queue.has_pending_callback());
    fail.set(false);
    queue.enqueue(|_| true).unwrap();
    fail.set(true);
    assert_eq!(queue.process(|| 0.0), Err("host request failed"));
    assert_eq!(queue.remaining_tasks(), 1);
    assert!(!queue.has_pending_callback());
    queue.clear();
    fail.set(false);
    queue.enqueue(|_| false).unwrap();
    queue.flush();
    assert_eq!(queue.remaining_tasks(), 0);
}
struct ReentrantWarningHost {
    host: Host,
    queue: Rc<RefCell<Option<xterm_core::task_queue::WeakTaskQueue<ReentrantWarningHost>>>>,
    warnings: Rc<RefCell<Vec<f64>>>,
}
impl Scheduler for ReentrantWarningHost {
    type Handle = usize;
    type Error = std::convert::Infallible;
    fn request(&self, kind: CallbackKind) -> Result<usize, Self::Error> {
        self.host.request(kind)
    }
    fn cancel(&self, id: usize) {
        self.host.cancel(id);
    }
    fn now(&self) -> f64 {
        self.host.now()
    }
    fn deadline_exceeded(&self, ms: f64) -> Result<(), Self::Error> {
        self.warnings.borrow_mut().push(ms);
        let queue = self.queue.borrow().as_ref().unwrap().upgrade().unwrap();
        queue.clear();
        queue.enqueue(|_| false).unwrap();
        Ok(())
    }
}
#[test]
fn warning_callback_can_clear_and_enqueue_without_borrowing_scheduler() {
    let host = Host::default();
    let clock = host.clock.clone();
    let dispatch = Rc::new(RefCell::new(None));
    let warnings = Rc::new(RefCell::new(Vec::new()));
    let queue = TaskQueue::idle(
        ReentrantWarningHost {
            host,
            queue: dispatch.clone(),
            warnings: warnings.clone(),
        },
        true,
    );
    *dispatch.borrow_mut() = Some(queue.downgrade());
    queue
        .enqueue(move |_| {
            clock.set(25.0);
            false
        })
        .unwrap();
    queue.process(|| 1.0).unwrap();
    assert_eq!(warnings.borrow().as_slice(), [24.0]);
    assert_eq!(queue.remaining_tasks(), 1);
    assert!(queue.has_pending_callback());
    queue.process(|| 100.0).unwrap();
    assert_eq!(queue.remaining_tasks(), 0);
}
struct WarningFailureHost(Host);
impl Scheduler for WarningFailureHost {
    type Handle = usize;
    type Error = &'static str;
    fn request(&self, kind: CallbackKind) -> Result<usize, Self::Error> {
        Ok(self.0.request(kind).unwrap())
    }
    fn cancel(&self, id: usize) {
        self.0.cancel(id);
    }
    fn now(&self) -> f64 {
        self.0.now()
    }
    fn deadline_exceeded(&self, _: f64) -> Result<(), Self::Error> {
        Err("log callback failed")
    }
}
#[test]
fn warning_failure_preserves_partial_index_and_allows_retry() {
    let host = Host::default();
    let clock = host.clock.clone();
    let queue = TaskQueue::idle(WarningFailureHost(host), true);
    queue
        .enqueue(move |_| {
            clock.set(25.0);
            false
        })
        .unwrap();
    queue.enqueue(|_| false).unwrap();
    assert_eq!(queue.process(|| 1.0), Err("log callback failed"));
    assert_eq!(queue.remaining_tasks(), 1);
    assert!(!queue.has_pending_callback());
    queue.flush();
    assert_eq!(queue.remaining_tasks(), 0);
}
#[test]
fn weak_dispatch_does_not_extend_lifetime() {
    let host = Host::default();
    let cancelled = host.cancelled.clone();
    let queue = TaskQueue::idle(host, true);
    queue.enqueue(|_| false).unwrap();
    let weak = queue.downgrade();
    let cloned = weak.clone();
    assert!(weak.upgrade().is_some());
    drop(queue);
    assert!(weak.upgrade().is_none());
    assert!(cloned.upgrade().is_none());
    assert_eq!(cancelled.borrow().as_slice(), [1]);
}
#[test]
fn nan_duration_and_deadline_preserve_js_comparisons() {
    for nan_clock in [true, false] {
        let host = Host::default();
        let clock = host.clock.clone();
        let queue = TaskQueue::idle(host, true);
        queue
            .enqueue(move |_| {
                if nan_clock {
                    clock.set(f64::NAN);
                }
                false
            })
            .unwrap();
        queue.enqueue(|_| false).unwrap();
        queue
            .process(|| if nan_clock { 0.0 } else { f64::NAN })
            .unwrap();
        assert_eq!(queue.remaining_tasks(), 0);
        assert!(!queue.has_pending_callback());
    }
}
#[test]
fn callback_panic_preserves_retry_index_and_pending_semantics() {
    for processing in [true, false] {
        let host = Host::default();
        let cancelled = host.cancelled.clone();
        let requested = host.requested.clone();
        let queue = TaskQueue::idle(host, true);
        let first = Cell::new(true);
        queue
            .enqueue(move |_| {
                assert!(!first.replace(false), "test callback failure");
                false
            })
            .unwrap();
        let result = catch_unwind(AssertUnwindSafe(|| {
            if processing {
                queue.process(|| 100.0).unwrap()
            } else {
                queue.flush()
            }
        }));
        assert!(result.is_err());
        assert_eq!(queue.remaining_tasks(), 1);
        assert_eq!(queue.has_pending_callback(), !processing);
        queue.enqueue(|_| false).unwrap();
        queue.flush();
        assert_eq!(requested.get(), if processing { 2 } else { 1 });
        assert_eq!(
            cancelled.borrow().as_slice(),
            if processing { [2] } else { [1] }
        );
    }
}
