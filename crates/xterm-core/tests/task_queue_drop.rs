use std::{
    cell::{Cell, RefCell},
    panic::{catch_unwind, AssertUnwindSafe},
    rc::Rc,
};
use xterm_core::task_queue::{CallbackKind, Scheduler, TaskQueue};

#[derive(Clone, Default)]
struct Host {
    next: Rc<Cell<usize>>,
    trace: Rc<RefCell<Vec<String>>>,
}
impl Scheduler for Host {
    type Handle = usize;
    type Error = std::convert::Infallible;

    fn request(&self, _: CallbackKind) -> Result<usize, Self::Error> {
        let id = self.next.get() + 1;
        self.next.set(id);
        self.trace.borrow_mut().push(format!("request:{id}"));
        Ok(id)
    }
    fn cancel(&self, id: usize) {
        self.trace.borrow_mut().push(format!("cancel:{id}"));
    }
    fn callback_fired(&self, id: usize) {
        self.trace.borrow_mut().push(format!("fired:{id}"));
    }
    fn now(&self) -> f64 {
        0.0
    }
    fn deadline_exceeded(&self, _: f64) -> Result<(), Self::Error> {
        panic!("unexpected overrun");
    }
}

struct OnDrop(Option<Box<dyn FnOnce()>>);
impl Drop for OnDrop {
    fn drop(&mut self) {
        self.0.take().unwrap()();
    }
}

fn enqueue_drop(queue: &TaskQueue<Host>, action: impl FnOnce() + 'static) {
    let capture = OnDrop(Some(Box::new(action)));
    queue
        .enqueue(move |_| {
            let _retain = &capture;
            false
        })
        .unwrap();
}

#[test]
fn clear_drops_captures_after_reset_and_preserves_reentrant_enqueue() {
    let host = Host::default();
    let trace = host.trace.clone();
    let queue = TaskQueue::idle(host, true);
    let weak = queue.downgrade();
    let count = Rc::new(Cell::new(0));
    let called = count.clone();
    enqueue_drop(&queue, move || {
        let queue = weak.upgrade().unwrap();
        assert_eq!(queue.remaining_tasks(), 0);
        assert!(!queue.has_pending_callback());
        queue
            .enqueue(move |_| {
                called.set(called.get() + 1);
                false
            })
            .unwrap();
    });
    queue.clear();
    assert_eq!(queue.remaining_tasks(), 1);
    assert!(queue.has_pending_callback());
    assert_eq!(count.get(), 0);
    assert_eq!(
        trace.borrow().as_slice(),
        ["request:1", "cancel:1", "request:2"]
    );
    queue.flush().unwrap();
    assert_eq!(count.get(), 1);
    assert_eq!(queue.remaining_tasks(), 0);
    assert!(!queue.has_pending_callback());
    assert_eq!(trace.borrow().last().unwrap(), "cancel:2");
}

#[test]
fn capture_can_clear_recursively_without_holding_the_task_borrow() {
    let host = Host::default();
    let trace = host.trace.clone();
    let queue = TaskQueue::idle(host, true);
    let weak = queue.downgrade();
    let dropped = Rc::new(Cell::new(0));
    let seen = dropped.clone();
    enqueue_drop(&queue, move || {
        let queue = weak.upgrade().unwrap();
        queue.clear();
        seen.set(seen.get() + 1);
        queue.enqueue(|_| false).unwrap();
        queue.clear();
    });
    queue.clear();
    assert_eq!(dropped.get(), 1);
    assert_eq!(queue.remaining_tasks(), 0);
    assert!(!queue.has_pending_callback());
    assert_eq!(
        trace.borrow().as_slice(),
        ["request:1", "cancel:1", "request:2", "cancel:2"]
    );
}

#[test]
fn completed_flush_and_dispatch_release_captures_outside_borrows() {
    for scheduled in [false, true] {
        let host = Host::default();
        let trace = host.trace.clone();
        let queue = TaskQueue::idle(host, true);
        // A processed prefix makes resetting the index observable to Drop.
        queue.enqueue(|_| false).unwrap();
        let weak = queue.downgrade();
        enqueue_drop(&queue, move || {
            let queue = weak.upgrade().unwrap();
            assert_eq!(queue.remaining_tasks(), 0);
            assert!(!queue.has_pending_callback());
            queue.enqueue(|_| false).unwrap();
        });
        if scheduled {
            queue.process(|| 100.0).unwrap();
        } else {
            queue.flush().unwrap();
        }
        assert_eq!(queue.remaining_tasks(), 1);
        assert!(queue.has_pending_callback());
        assert_eq!(
            trace.borrow().as_slice(),
            [
                "request:1",
                if scheduled { "fired:1" } else { "cancel:1" },
                "request:2"
            ]
        );
        queue.clear();
        assert_eq!(queue.remaining_tasks(), 0);
        assert_eq!(trace.borrow().last().unwrap(), "cancel:2");
    }
}

#[test]
fn capture_panic_propagates_with_empty_reusable_queue() {
    let host = Host::default();
    let trace = host.trace.clone();
    let queue = TaskQueue::idle(host, true);
    enqueue_drop(&queue, || panic!("capture disposal failed"));
    let error = catch_unwind(AssertUnwindSafe(|| queue.clear())).unwrap_err();
    assert_eq!(
        error.downcast_ref::<&str>(),
        Some(&"capture disposal failed")
    );
    assert_eq!(queue.remaining_tasks(), 0);
    assert!(!queue.has_pending_callback());
    queue.enqueue(|_| false).unwrap();
    queue.flush().unwrap();
    assert_eq!(queue.remaining_tasks(), 0);
    assert_eq!(
        trace.borrow().as_slice(),
        ["request:1", "cancel:1", "request:2", "cancel:2"]
    );
}

#[test]
fn capture_enqueue_before_panic_survives_unwinding() {
    let queue = TaskQueue::idle(Host::default(), true);
    let weak = queue.downgrade();
    enqueue_drop(&queue, move || {
        let queue = weak.upgrade().unwrap();
        queue.enqueue(|_| false).unwrap();
        panic!("capture failed after enqueue");
    });
    let error = catch_unwind(AssertUnwindSafe(|| queue.clear())).unwrap_err();
    assert_eq!(
        error.downcast_ref::<&str>(),
        Some(&"capture failed after enqueue")
    );
    assert_eq!(queue.remaining_tasks(), 1);
    assert!(queue.has_pending_callback());
    queue.flush().unwrap();
    assert_eq!(queue.remaining_tasks(), 0);
    assert!(!queue.has_pending_callback());
}
