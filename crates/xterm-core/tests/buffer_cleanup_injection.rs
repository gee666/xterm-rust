// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    convert::Infallible,
    rc::Rc,
};
use xterm_core::{
    buffer::{cleanup::CleanupError, Buffer, BufferCleanup, BufferError},
    options::OptionsService,
    task_queue::{CallbackKind, Scheduler, TaskQueue},
};
#[derive(Clone, Default)]
struct Host {
    requests: Rc<Cell<usize>>,
    cancels: Rc<Cell<usize>>,
}
impl Scheduler for Host {
    type Handle = usize;
    type Error = Infallible;
    fn request(&self, _: CallbackKind) -> Result<usize, Infallible> {
        let id = self.requests.get() + 1;
        self.requests.set(id);
        Ok(id)
    }
    fn cancel(&self, _: usize) {
        self.cancels.set(self.cancels.get() + 1);
    }
    fn now(&self) -> f64 {
        0.0
    }
    fn deadline_exceeded(&self, _: f64) -> Result<(), Infallible> {
        panic!("fixed cleanup clock cannot overrun");
    }
}
fn buffer() -> Rc<RefCell<Buffer>> {
    let mut buffer = Buffer::new(true, 10, 3, Rc::new(OptionsService::default())).unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    Rc::new(RefCell::new(buffer))
}
#[test]
fn configured_host_queue_dispatches_the_actual_cleanup_task() {
    let buffer = buffer();
    let host = Host::default();
    let queue = TaskQueue::idle(host.clone(), true);
    let adapter = BufferCleanup::with_queue(&buffer, queue.clone()).unwrap();
    Buffer::resize_shared(&buffer, 3, 3).unwrap();
    assert_eq!(host.requests.get(), 1);
    queue.process(|| 1000.0).unwrap();
    assert!(adapter.take_error().is_none());
    assert!(!buffer.borrow().has_pending_cleanup());
    assert!(!adapter.has_pending_callback());
}
#[test]
fn injected_busy_queue_is_rejected_without_cancelling_other_work() {
    let buffer = buffer();
    let host = Host::default();
    let queue = TaskQueue::idle(host.clone(), true);
    let ran = Rc::new(Cell::new(false));
    let mark = ran.clone();
    queue
        .enqueue(move |_| {
            mark.set(true);
            false
        })
        .unwrap();
    assert!(matches!(
        BufferCleanup::with_queue(&buffer, queue.clone()),
        Err(CleanupError::Buffer(BufferError::CleanupQueueBusy))
    ));
    assert_eq!(host.cancels.get(), 0);
    queue.process(|| 1000.0).unwrap();
    assert!(ran.get());
    let _adapter = BufferCleanup::with_queue(&buffer, queue).unwrap();
}
#[test]
fn root_buffer_disposal_is_idempotent_even_after_late_marker_registration() {
    let buffer = buffer();
    buffer.borrow_mut().dispose().unwrap();
    let marker = buffer.borrow_mut().add_marker(0);
    buffer.borrow_mut().dispose().unwrap();
    assert!(!marker.is_disposed());
    assert_eq!(marker.line(), 0);
    assert_eq!(buffer.borrow().markers.borrow().len(), 1);
}
