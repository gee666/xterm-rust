// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use xterm_core::input::write_buffer::{WriteData, WriteError};
fn marker() -> WriteError {
    WriteError::external(std::io::Error::other("original cancel failure"))
}
#[test]
fn failed_reschedule_retains_token_and_does_not_accept_new_data() {
    let (queue, host, stack, _) = queue();
    queue.write(WriteData::text("first"), None).unwrap();
    queue.flush_sync().unwrap();
    let error = marker();
    *host.0.cancel_error.borrow_mut() = Some(error.clone());
    assert_eq!(queue.write(WriteData::text("second"), None), Err(error));
    assert!(queue.has_scheduled_write());
    assert_eq!(queue.buffered_chunks(), 0);
    assert_eq!(queue.pending_units(), 0.0);
    host.0.cancel_error.borrow_mut().take();
    host.tasks();
    assert_eq!(*stack.borrow(), [WriteData::text("first")]);
}
#[test]
fn explicit_dispose_keeps_original_error_but_clears_other_registered_resources() {
    let (queue, host, stack, calls) = queue();
    queue
        .write(WriteData::text("a"), callback(&calls, "a"))
        .unwrap();
    let error = marker();
    *host.0.cancel_error.borrow_mut() = Some(error.clone());
    assert_eq!(queue.try_dispose(), Err(error));
    assert!(queue.is_disposed());
    assert_eq!(queue.pending_units(), 0.0);
    assert_eq!(queue.buffered_chunks(), 0);
    assert!(queue.has_scheduled_write());
    assert!(host.0.errors.borrow().is_empty());
    queue.try_dispose().unwrap();
    host.0.cancel_error.borrow_mut().take();
    host.tasks();
    assert!(stack.borrow().is_empty());
    assert!(calls.borrow().is_empty());
}
#[test]
fn infallible_dispose_reports_original_host_error_instead_of_swallowing() {
    let (queue, host, _, _) = queue();
    queue.write(WriteData::text("a"), None).unwrap();
    let error = marker();
    *host.0.cancel_error.borrow_mut() = Some(error.clone());
    queue.dispose();
    assert_eq!(*host.0.errors.borrow(), [error]);
    host.0.cancel_error.borrow_mut().take();
    host.tasks();
}
#[test]
fn last_owner_drop_reports_cancellation_failure_without_retaining_queue() {
    let (queue, host, _, _) = queue();
    let weak = queue.downgrade();
    queue.write(WriteData::text("a"), None).unwrap();
    let error = marker();
    *host.0.cancel_error.borrow_mut() = Some(error.clone());
    drop(queue);
    assert!(weak.upgrade().is_none());
    assert_eq!(*host.0.errors.borrow(), [error]);
    host.tasks();
}
