// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use super::*;
use crate::{
    buffer::cleanup::CleanupError,
    headless::{NativeWritePump, Terminal},
    log_service::{LogError, LogRecord, LogSink},
};
use std::{cell::Cell, error::Error, fmt};

#[test]
fn closed_real_host_returns_the_same_typed_scheduling_error() {
    let terminal = Terminal::new(80, 4, 8).unwrap();
    let buffer = terminal.with_state(|state| state.active_buffer()).unwrap();
    let logger = terminal
        .with_state(|state| Rc::downgrade(&state.logger))
        .unwrap();
    let pump = NativeWritePump::with_clock(|| 100.0);
    let owner = attach(&buffer, &pump.scheduler(), &logger).unwrap();
    let adapter = owner.downcast_ref::<Rc<Adapter>>().unwrap().clone();
    drop(pump);
    let error = Buffer::resize_shared(&buffer, 20, 4).unwrap_err();
    let BufferError::Dispatch(EmitError::Callback(dispatched)) = error else {
        panic!("expected actual host failure");
    };
    let Some(CleanupError::ScheduledCallback(retained)) = adapter.take_error() else {
        panic!("expected diagnostic of same failure");
    };
    assert_eq!(retained.to_string(), "native write pump is closed");
    let retained: Rc<dyn Error> = retained;
    assert!(Rc::ptr_eq(&dispatched, &retained));
    assert!(buffer.borrow().has_pending_cleanup());
    assert!(!adapter.has_pending_callback());
}

#[derive(Debug)]
struct RejectedWarning;
impl fmt::Display for RejectedWarning {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("warning observer failed")
    }
}
impl Error for RejectedWarning {}
struct Reject(Rc<RejectedWarning>);
impl LogSink for Reject {
    fn log(&self, _: LogRecord) -> Result<(), LogError> {
        Err(LogError::Host(self.0.clone()))
    }
}

#[test]
fn actual_host_surfaces_logger_failure_without_scheduling_successful_retry() {
    let terminal = Terminal::new(80, 250, 8).unwrap();
    let buffer = terminal.with_state(|state| state.active_buffer()).unwrap();
    let logger = terminal
        .with_state(|state| Rc::clone(&state.logger))
        .unwrap();
    let failure = Rc::new(RejectedWarning);
    logger.set_console_sink(Rc::new(Reject(failure.clone())));
    let time = Rc::new(Cell::new(100.0));
    let clock = time.clone();
    let pump = NativeWritePump::with_clock(move || clock.replace(clock.get() + 100.0));
    let owner = attach(&buffer, &pump.scheduler(), &Rc::downgrade(&logger)).unwrap();
    Buffer::resize_shared(&buffer, 20, 250).unwrap();
    let errors = pump.poll().unwrap_err();
    assert_eq!(errors.len(), 1);
    let mut current: Option<&(dyn Error + 'static)> = Some(&errors[0]);
    let mut same_object = false;
    while let Some(error) = current {
        if let Some(actual) = error.downcast_ref::<RejectedWarning>() {
            same_object |= std::ptr::eq(actual, failure.as_ref());
        }
        current = error.source();
    }
    assert!(
        same_object,
        "original logger error identity must survive every wrapper"
    );
    assert_eq!(
        pump.pending_tasks(),
        0,
        "failed warning must not schedule a hidden retry"
    );
    assert!(buffer.borrow().has_pending_cleanup());
    drop(owner);
}
