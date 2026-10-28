// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::input::write_buffer::{
    WriteAction, WriteBuffer, WriteData, WriteError, DISCARD_WATERMARK,
};

#[test]
fn exact_discard_guard_and_utf16_length() {
    let (queue, host, _, _) = queue();
    queue
        .write(WriteData::Bytes(vec![0; DISCARD_WATERMARK]), None)
        .unwrap();
    assert_eq!(queue.pending_units(), 50_000_000.0);
    queue.write(WriteData::text("😀"), None).unwrap();
    assert_eq!(queue.pending_units(), 50_000_002.0);
    assert_eq!(
        queue.write(WriteData::text(""), None),
        Err(WriteError::Discarded)
    );
    assert_eq!(
        WriteError::Discarded.to_string(),
        "write data discarded, use flow control to avoid losing data"
    );
    assert_eq!(host.pending_tasks(), 1);
    queue.dispose();
}
#[test]
fn slice_boundary_and_compaction_are_strict() {
    for (cut, retained, offset) in [(50, 53, 50), (51, 2, 0)] {
        let host = Host::default();
        host.advance(100.0);
        let clock = host.clone();
        let count = Rc::new(Cell::new(0));
        let counter = count.clone();
        let queue = WriteBuffer::new(host.clone(), move |_, _, _| {
            counter.set(counter.get() + 1);
            if counter.get() == cut {
                clock.advance(12.0);
            }
            Ok(WriteAction::Complete)
        });
        let events = Rc::new(Cell::new(0));
        let output = events.clone();
        let weak = queue.downgrade();
        queue.on_write_parsed().subscribe(move |_| {
            let queue = weak.upgrade().unwrap();
            if output.get() == 0 {
                assert!(queue.has_scheduled_write());
            }
            output.set(output.get() + 1);
        });
        for _ in 0..53 {
            queue.write(WriteData::text("a"), None).unwrap();
        }
        assert!(host.task().unwrap());
        assert_eq!(count.get(), cut);
        assert_eq!(queue.buffered_chunks(), retained);
        assert_eq!(queue.buffer_offset(), offset);
        assert_eq!(queue.pending_units(), (53 - cut) as f64);
        assert_eq!(events.get(), 1);
        host.tasks();
        assert_eq!(count.get(), 53);
        assert_eq!(events.get(), 2);
    }
}
#[test]
fn continuation_below_or_at_budget_preserves_boolean_and_callbacks() {
    for elapsed in [11.999, 12.0] {
        let host = Host::default();
        host.advance(100.0);
        let log = Rc::new(RefCell::new(Vec::new()));
        let output = log.clone();
        let first = Rc::new(Cell::new(true));
        let flag = first.clone();
        let queue = WriteBuffer::new(host.clone(), move |_, _, result| {
            output.borrow_mut().push(result);
            if flag.replace(false) {
                Ok(WriteAction::Pending(Box::pin(async { Ok(false) })))
            } else {
                Ok(WriteAction::Complete)
            }
        });
        let calls = Calls::default();
        queue
            .write(WriteData::text("a"), callback(&calls, "a"))
            .unwrap();
        queue
            .write(WriteData::text("b"), callback(&calls, "b"))
            .unwrap();
        host.task().unwrap();
        assert_eq!(queue.pending_units(), 2.0);
        assert!(calls.borrow().is_empty());
        host.advance(elapsed);
        host.microtasks();
        if elapsed >= 12.0 {
            assert_eq!(*log.borrow(), [Some(true)]);
            assert!(queue.has_scheduled_write());
            host.tasks();
        }
        assert_eq!(*log.borrow(), [Some(true), Some(false), Some(false)]);
        assert_eq!(*calls.borrow(), ["a", "b"]);
        assert_eq!(queue.pending_units(), 0.0);
    }
}
#[test]
fn rejection_reports_original_error_and_resumes_false() {
    let host = Host::default();
    let count = Rc::new(Cell::new(0));
    let c = count.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, _, result| {
        c.set(c.get() + 1);
        if c.get() == 1 {
            Ok(WriteAction::Pending(Box::pin(async {
                Err(WriteError::Action("reject".into()))
            })))
        } else {
            assert_eq!(result, Some(false));
            Ok(WriteAction::Complete)
        }
    });
    queue.write(WriteData::text("a"), None).unwrap();
    host.task().unwrap();
    host.microtasks();
    assert_eq!(
        *host.0.errors.borrow(),
        [WriteError::Action("reject".into())]
    );
    assert_eq!(count.get(), 1);
    // The uncaught error is queued before the resolved-false continuation.
    host.microtasks();
    assert_eq!(count.get(), 2);
    assert_eq!(queue.pending_units(), 0.0);
}
#[test]
fn disposing_actual_pending_handler_prevents_continuation() {
    let host = Host::default();
    let calls = Rc::new(Cell::new(0));
    let c = calls.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, _, _| {
        c.set(c.get() + 1);
        Ok(WriteAction::Pending(Box::pin(async { Ok(true) })))
    });
    queue.write(WriteData::text("a"), None).unwrap();
    host.task().unwrap();
    assert_eq!(calls.get(), 1);
    queue.dispose();
    host.microtasks();
    host.tasks();
    assert_eq!(calls.get(), 1);
    assert_eq!(queue.pending_units(), 0.0);
}
#[test]
fn synchronous_empty_text_stops_but_empty_bytes_do_not() {
    for bytes in [false, true] {
        let (queue, host, stack, calls) = queue();
        queue
            .write(
                if bytes {
                    WriteData::Bytes(vec![])
                } else {
                    WriteData::text("")
                },
                callback(&calls, "empty"),
            )
            .unwrap();
        queue.write_sync(WriteData::text("x"), None).unwrap();
        if bytes {
            assert_eq!(
                *stack.borrow(),
                [WriteData::Bytes(vec![]), WriteData::text("x")]
            );
            assert_eq!(*calls.borrow(), ["empty"]);
        } else {
            assert!(stack.borrow().is_empty());
            assert!(calls.borrow().is_empty());
            assert_eq!(queue.buffered_chunks(), 1);
        }
        host.tasks();
        assert_eq!(queue.pending_units(), 0.0);
    }
}
#[test]
fn action_and_callback_errors_propagate_partial_state() {
    let host = Host::default();
    let failed_queue = WriteBuffer::new(host.clone(), |_, _, _| {
        Err(WriteError::Action("action".into()))
    });
    failed_queue.write(WriteData::text("a"), None).unwrap();
    assert_eq!(host.task(), Err(WriteError::Action("action".into())));
    assert_eq!(failed_queue.pending_units(), 1.0);
    assert_eq!(failed_queue.buffer_offset(), 0);
    let (queue, host, stack, _) = queue();
    queue
        .write(
            WriteData::text("a"),
            Some(Rc::new(|_| Err(WriteError::Callback("callback".into())))),
        )
        .unwrap();
    assert_eq!(host.task(), Err(WriteError::Callback("callback".into())));
    assert_eq!(stack.borrow().len(), 1);
    assert_eq!(queue.pending_units(), 1.0);
    assert_eq!(queue.buffer_offset(), 0);
}
#[test]
fn callbacks_reenter_without_borrows_or_lost_fifo() {
    let (queue, host, stack, calls) = queue();
    let log = calls.clone();
    queue
        .write(
            WriteData::text("a"),
            Some(Rc::new(move |queue| {
                log.borrow_mut().push("a".into());
                queue.write(WriteData::text("c"), callback(&log, "c"))
            })),
        )
        .unwrap();
    queue
        .write(WriteData::text("b"), callback(&calls, "b"))
        .unwrap();
    host.tasks();
    assert_eq!(*stack.borrow(), ["a", "b", "c"].map(WriteData::text));
    assert_eq!(*calls.borrow(), ["a", "b", "c"]);
}
#[test]
fn dropping_final_owner_cancels_timer_and_weak_handle_expires() {
    let (queue, host, _, _) = queue();
    let weak = queue.downgrade();
    queue.write(WriteData::text("a"), None).unwrap();
    drop(queue);
    assert!(weak.upgrade().is_none());
    assert_eq!(host.pending_tasks(), 0);
}
