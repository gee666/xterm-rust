use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::task_queue::{CallbackKind, DebouncedIdleTask, Scheduler, TaskQueue};

#[derive(Clone, Default)]
struct Host {
    clock: Rc<Cell<f64>>,
    events: Rc<RefCell<Vec<String>>>,
    token: Cell<usize>,
}
impl Scheduler for Host {
    type Handle = usize;
    type Error = std::convert::Infallible;
    fn request(&self, kind: CallbackKind) -> Result<usize, Self::Error> {
        self.token.set(self.token.get() + 1);
        self.events
            .borrow_mut()
            .push(format!("request:{}:{kind:?}", self.token.get()));
        Ok(self.token.get())
    }
    fn cancel(&self, id: usize) {
        self.events.borrow_mut().push(format!("cancel:{id}"));
    }
    fn now(&self) -> f64 {
        self.clock.get()
    }
    fn deadline_exceeded(&self, ms: f64) -> Result<(), Self::Error> {
        self.events.borrow_mut().push(format!("warn:{ms}"));
        Ok(())
    }
}
#[test]
fn fifo_repetition_flush_and_cancel() {
    let host = Host::default();
    let events = host.events.clone();
    let queue = TaskQueue::new(host, CallbackKind::Priority);
    let count = Rc::new(Cell::new(0));
    let task_count = count.clone();
    queue
        .enqueue(move |_| {
            task_count.set(task_count.get() + 1);
            task_count.get() < 3
        })
        .unwrap();
    let seen = Rc::new(Cell::new(0));
    let task_seen = seen.clone();
    queue
        .enqueue(move |_| {
            task_seen.set(1);
            false
        })
        .unwrap();
    assert_eq!(events.borrow().as_slice(), ["request:1:Priority"]);
    queue.flush();
    assert_eq!(count.get(), 3);
    assert_eq!(seen.get(), 1);
    assert_eq!(
        events.borrow().as_slice(),
        ["request:1:Priority", "cancel:1"]
    );
    assert_eq!(queue.remaining_tasks(), 0);
    queue.clear();
    queue.flush();
    assert_eq!(events.borrow().len(), 2);
}
#[test]
fn deadline_equality_and_final_task_reschedule() {
    let host = Host::default();
    let events = host.events.clone();
    let queue = TaskQueue::idle(host, true);
    queue.enqueue(|_| false).unwrap();
    queue.process(|| 1.5).unwrap();
    assert!(!queue.has_pending_callback());
    assert_eq!(events.borrow().len(), 1);
    queue.enqueue(|_| false).unwrap();
    queue.process(|| 1.499).unwrap();
    assert!(queue.has_pending_callback());
    assert_eq!(queue.remaining_tasks(), 0);
    assert_eq!(
        events.borrow().as_slice(),
        ["request:1:Idle", "request:2:Idle", "request:3:Idle"]
    );
    queue.process(|| 0.0).unwrap();
    assert!(!queue.has_pending_callback());
}
#[test]
fn warning_threshold_and_javascript_rounding() {
    for (duration, expected) in [
        (21.0, None),
        (21.5, Some("warn:20")),
        (22.5, Some("warn:21")),
    ] {
        let host = Host::default();
        let clock = host.clock.clone();
        let events = host.events.clone();
        let queue = TaskQueue::idle(host, true);
        queue
            .enqueue(move |_| {
                clock.set(duration);
                false
            })
            .unwrap();
        queue.process(|| 1.0).unwrap();
        let warnings: Vec<_> = events
            .borrow()
            .iter()
            .filter(|s| s.starts_with("warn"))
            .cloned()
            .collect();
        assert_eq!(
            warnings,
            expected.into_iter().map(str::to_owned).collect::<Vec<_>>()
        );
    }
}
#[test]
fn longest_duration_clock_regression_and_priority_budget() {
    let host = Host::default();
    let clock = host.clock.clone();
    let queue = TaskQueue::idle(host, false);
    for delta in [10.0, -5.0, 1.0] {
        let clock = clock.clone();
        queue
            .enqueue(move |_| {
                clock.set(clock.get() + delta);
                false
            })
            .unwrap();
    }
    queue.process_priority().unwrap();
    assert_eq!(queue.remaining_tasks(), 2);
    queue.process_priority().unwrap();
    assert_eq!(queue.remaining_tasks(), 0);
    assert!(!queue.has_pending_callback());
}
#[test]
fn reentrant_enqueue_clear_and_recursive_flush() {
    let queue = TaskQueue::idle(Host::default(), true);
    let seen = Rc::new(Cell::new(0));
    let task_seen = seen.clone();
    queue
        .enqueue(move |queue| {
            let task_seen = task_seen.clone();
            queue
                .enqueue(move |_| {
                    task_seen.set(task_seen.get() + 1);
                    false
                })
                .unwrap();
            false
        })
        .unwrap();
    queue.flush();
    assert_eq!(seen.get(), 1);
    let task_seen = seen.clone();
    queue
        .enqueue(move |queue| {
            queue.clear();
            let task_seen = task_seen.clone();
            queue
                .enqueue(move |_| {
                    task_seen.set(99);
                    false
                })
                .unwrap();
            false
        })
        .unwrap();
    queue.flush();
    assert_eq!(seen.get(), 1); // upstream advances the reset index
    let entered = Cell::new(false);
    queue
        .enqueue(move |queue| {
            if !entered.replace(true) {
                queue.flush();
            }
            false
        })
        .unwrap();
    queue.flush();
    assert_eq!(queue.remaining_tasks(), 0);
}
#[test]
fn debounce_replaces_disposes_and_allows_reuse() {
    let host = Host::default();
    let events = host.events.clone();
    let task = DebouncedIdleTask::new(host, false);
    task.set(|_| panic!("replaced task must not run")).unwrap();
    let calls = Rc::new(Cell::new(0));
    let callback_calls = calls.clone();
    task.set(move |_| {
        callback_calls.set(callback_calls.get() + 1);
        callback_calls.get() < 2
    })
    .unwrap();
    task.flush();
    assert_eq!(calls.get(), 2);
    task.set(|_| panic!("disposed task must not run")).unwrap();
    task.dispose();
    task.flush();
    task.set(|_| false).unwrap();
    task.process_priority().unwrap();
    assert_eq!(
        events
            .borrow()
            .iter()
            .filter(|s| s.starts_with("cancel"))
            .count(),
        3
    );
}
#[test]
fn last_handle_drop_cancels_only_once() {
    let host = Host::default();
    let events = host.events.clone();
    let queue = TaskQueue::idle(host, true);
    queue.enqueue(|_| false).unwrap();
    let cloned = queue.clone();
    drop(queue);
    assert_eq!(events.borrow().len(), 1);
    drop(cloned);
    assert_eq!(events.borrow().as_slice(), ["request:1:Idle", "cancel:1"]);
}
