// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use std::{cell::Cell, rc::Rc};
use xterm_core::input::write_buffer::{WriteAction, WriteBuffer, WriteData, WriteError};
#[test]
fn host_request_error_is_recoverable_without_queued_data_or_held_borrow() {
    let (queue, host, stack, _) = queue();
    host.0.fail_requests.set(true);
    assert_eq!(
        queue.write(WriteData::text("a"), None),
        Err(WriteError::Schedule("host request failed".into()))
    );
    assert_eq!(queue.pending_units(), 0.0);
    assert_eq!(queue.buffered_chunks(), 0);
    host.0.fail_requests.set(false);
    queue.write(WriteData::text("b"), None).unwrap();
    host.tasks();
    assert_eq!(*stack.borrow(), [WriteData::text("b")]);
}
#[test]
fn sync_throw_retains_source_in_progress_state() {
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let queue = WriteBuffer::new(Host::default(), move |_, _, _| {
        count.set(count.get() + 1);
        Err(WriteError::Action("sync throw".into()))
    });
    assert_eq!(
        queue.write_sync(WriteData::text("a"), None),
        Err(WriteError::Action("sync throw".into()))
    );
    queue.write_sync(WriteData::text("b"), None).unwrap();
    queue.flush_sync().unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(queue.pending_units(), 2.0);
    assert_eq!(queue.buffered_chunks(), 1);
}
#[test]
fn sync_flush_called_from_action_does_not_recurse() {
    let queue = WriteBuffer::new(Host::default(), |queue, _, _| {
        queue.flush_sync()?;
        Ok(WriteAction::Complete)
    });
    queue.write_sync(WriteData::text("a"), None).unwrap();
    assert_eq!(queue.pending_units(), 0.0);
}
#[test]
fn flush_stops_at_empty_text_then_clears_remaining_callbacks() {
    let (queue, host, stack, calls) = queue();
    queue
        .write(WriteData::text(""), callback(&calls, "empty"))
        .unwrap();
    queue
        .write(WriteData::text("a"), callback(&calls, "a"))
        .unwrap();
    queue.flush_sync().unwrap();
    host.tasks();
    assert!(stack.borrow().is_empty());
    assert!(calls.borrow().is_empty());
    assert_eq!(queue.buffered_chunks(), 0);
}
#[test]
fn user_input_flag_is_retained_while_old_data_is_queued() {
    let (queue, host, stack, _) = queue();
    queue.write(WriteData::text("a"), None).unwrap();
    queue.handle_user_input();
    queue.write(WriteData::text("b"), None).unwrap();
    assert!(stack.borrow().is_empty());
    host.tasks();
    queue.write(WriteData::text("c"), None).unwrap();
    assert_eq!(*stack.borrow(), ["a", "b", "c"].map(WriteData::text));
    assert_eq!(host.pending_tasks(), 0);
}
