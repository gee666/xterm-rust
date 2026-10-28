// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    buffer::{cleanup::CleanupError, Buffer, BufferCleanup, BufferError},
    options::OptionsService,
    task_queue::{CallbackKind, Scheduler, TaskQueue},
};
#[derive(Debug)]
struct HostError(BufferError);
impl From<BufferError> for HostError {
    fn from(error: BufferError) -> Self {
        Self(error)
    }
}
impl std::fmt::Display for HostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl std::error::Error for HostError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.0)
    }
}

#[derive(Clone, Default)]
struct Host {
    requests: Rc<Cell<usize>>,
    cancels: Rc<Cell<usize>>,
}
impl Scheduler for Host {
    type Handle = usize;
    type Error = HostError;
    fn request(&self, _: CallbackKind) -> Result<usize, HostError> {
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
    fn deadline_exceeded(&self, _: f64) -> Result<(), HostError> {
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
fn injected_callback_failure_retains_task_and_stops_deadline_queries() {
    let buffer = buffer();
    let queue = TaskQueue::idle(Host::default(), true);
    let _adapter = BufferCleanup::with_queue(&buffer, queue.clone()).unwrap();
    Buffer::resize_shared(&buffer, 3, 3).unwrap();
    let held = buffer.borrow_mut();
    let queries = Cell::new(0);
    let error = queue
        .process(|| {
            queries.set(queries.get() + 1);
            1000.0
        })
        .unwrap_err();
    assert!(matches!(error.0, BufferError::BorrowedLine));
    assert_eq!(queries.get(), 1);
    assert_eq!(queue.remaining_tasks(), 1);
    assert!(!queue.has_pending_callback());
    drop(held);
    queue.enqueue(|_| false).unwrap();
    queue.process(|| 1000.0).unwrap();
    assert!(!buffer.borrow().has_pending_cleanup());
}
#[test]
fn cleanup_position_advances_before_missing_row_error() {
    let buffer = buffer();
    Buffer::resize_shared(&buffer, 3, 3).unwrap();
    let first = buffer.borrow().lines.get(0).unwrap();
    buffer.borrow().lines.set(0, None).unwrap();
    assert!(matches!(
        buffer.borrow_mut().poll_cleanup(),
        Err(BufferError::MissingLine(0))
    ));
    buffer.borrow().lines.set(0, Some(first.clone())).unwrap();
    assert!(buffer.borrow_mut().poll_cleanup().unwrap());
    assert!(
        first.borrow_mut().cleanup_memory(),
        "failed row must have been skipped on resume"
    );
    assert!(!buffer.borrow_mut().poll_cleanup().unwrap());
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

#[derive(Debug)]
struct OpaqueFailure(u8);
#[derive(Debug)]
struct OpaqueWrapper(Rc<OpaqueFailure>);
impl std::fmt::Display for OpaqueWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("opaque host threw")
    }
}
impl std::error::Error for OpaqueWrapper {}
#[derive(Default)]
struct OpaqueHost {
    requests: Rc<Cell<usize>>,
    cancels: Rc<Cell<usize>>,
    fail: Rc<Cell<bool>>,
}
impl Scheduler for OpaqueHost {
    type Handle = usize;
    type Error = OpaqueFailure;
    fn request(&self, _: CallbackKind) -> Result<usize, OpaqueFailure> {
        if self.fail.get() {
            Err(OpaqueFailure(1))
        } else {
            let next = self.requests.get() + 1;
            self.requests.set(next);
            Ok(next)
        }
    }
    fn cancel(&self, _: usize) {
        self.cancels.set(self.cancels.get() + 1);
    }
    fn now(&self) -> f64 {
        0.0
    }
    fn deadline_exceeded(&self, _: f64) -> Result<(), OpaqueFailure> {
        panic!("fixed test deadline cannot overrun")
    }
}
#[test]
fn opaque_non_error_host_values_preserve_identity_without_late_rethrow() {
    let buffer = buffer();
    let host = OpaqueHost::default();
    let fail = host.fail.clone();
    let cancels = host.cancels.clone();
    let queue = TaskQueue::idle(host, true);
    let adapter = BufferCleanup::with_queue_map(
        &buffer,
        queue.clone(),
        |_| OpaqueFailure(2),
        |error| xterm_core::event::EmitError::callback(OpaqueWrapper(error)),
    )
    .unwrap();
    fail.set(true);
    let error = Buffer::resize_shared(&buffer, 3, 3).unwrap_err();
    let BufferError::Dispatch(error) = error else {
        panic!("expected actual callback object");
    };
    let raw = error.callback_ref::<OpaqueWrapper>().unwrap().0.clone();
    assert_eq!(raw.0, 1);
    fail.set(false);
    adapter.retry_pending().unwrap();
    adapter.process_idle(|| 1000.0).unwrap();
    let Some(CleanupError::ScheduledCallback(recorded)) = adapter.take_error() else {
        panic!("missing diagnostic payload");
    };
    assert!(Rc::ptr_eq(&raw, &recorded));
    Buffer::resize_shared(&buffer, 10, 3).unwrap();
    Buffer::resize_shared(&buffer, 3, 3).unwrap();
    let held = buffer.borrow_mut();
    let queries = Cell::new(0);
    let error = queue
        .process(|| {
            queries.set(queries.get() + 1);
            1000.0
        })
        .unwrap_err();
    assert_eq!(error.0, 2);
    assert_eq!(queries.get(), 1);
    assert_eq!(queue.remaining_tasks(), 1);
    drop(held);
    assert!(adapter.take_error().is_none());
    assert_eq!(cancels.get(), 0);
}
