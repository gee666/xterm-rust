// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use std::{cell::Cell, rc::Rc};
use xterm_core::input::write_buffer::{WriteAction, WriteBuffer, WriteCallback, WriteData};

fn data(text: &str, bytes: bool) -> WriteData {
    if bytes {
        WriteData::Bytes(text.as_bytes().to_vec())
    } else {
        WriteData::text(text)
    }
}
fn completion() -> (Rc<Cell<bool>>, Option<WriteCallback<Host>>) {
    let done = Rc::new(Cell::new(false));
    let output = done.clone();
    (
        done,
        Some(Rc::new(move |_| {
            output.set(true);
            Ok(())
        })),
    )
}
fn inputs(kinds: [bool; 5]) {
    let (queue, host, stack, calls) = queue();
    let (done, completed) = completion();
    for (i, text) in ["a._", "b.x", "c._", "d.x", "e"].iter().enumerate() {
        let cb = match i {
            1 => callback(&calls, "b"),
            3 => callback(&calls, "d"),
            4 => completed.clone(),
            _ => None,
        };
        queue.write(data(text, kinds[i]), cb).unwrap();
    }
    assert!(stack.borrow().is_empty());
    host.tasks();
    assert_eq!(
        *stack.borrow(),
        ["a._", "b.x", "c._", "d.x", "e"]
            .iter()
            .enumerate()
            .map(|(i, s)| data(s, kinds[i]))
            .collect::<Vec<_>>()
    );
    assert_eq!(*calls.borrow(), ["b", "d"]);
    assert!(done.get());
}
#[test]
fn string() {
    inputs([false; 5]);
}
#[test]
fn bytes() {
    inputs([true; 5]);
}
#[test]
fn string_bytes_mixed() {
    inputs([false, false, true, true, true]);
}
#[test]
fn write_callback_for_empty_chunks() {
    let (queue, host, stack, calls) = queue();
    for (i, (text, bytes)) in [("a", false), ("", false), ("c", true), ("", true)]
        .iter()
        .enumerate()
    {
        queue
            .write(
                data(text, *bytes),
                callback(&calls, ["a", "b", "c", "d"][i]),
            )
            .unwrap();
    }
    let (done, completed) = completion();
    queue.write(WriteData::text("e"), completed).unwrap();
    host.tasks();
    assert!(done.get());
    assert_eq!(
        *stack.borrow(),
        [
            data("a", false),
            data("", false),
            data("c", true),
            data("", true),
            data("e", false)
        ]
    );
    assert_eq!(*calls.borrow(), ["a", "b", "c", "d"]);
}
fn before_sync(queue: &WriteBuffer<Host>, calls: &Calls) {
    for s in ["a", "b", "c"] {
        queue.write(WriteData::text(s), callback(calls, s)).unwrap();
    }
}
fn after_sync(
    queue: &WriteBuffer<Host>,
    host: &Host,
    stack: &Stack,
    calls: &Calls,
    previous: &[&str],
    expected_callbacks: &[&str],
) {
    queue
        .write(WriteData::text("x"), callback(calls, "x"))
        .unwrap();
    let (done, completed) = completion();
    queue.write(WriteData::text(""), completed).unwrap();
    host.tasks();
    assert!(done.get());
    let mut expected: Vec<_> = previous.iter().map(|s| WriteData::text(s)).collect();
    expected.extend([WriteData::text("x"), WriteData::text("")]);
    assert_eq!(*stack.borrow(), expected);
    let mut cb: Vec<_> = expected_callbacks.iter().map(|s| s.to_string()).collect();
    cb.push("x".into());
    assert_eq!(*calls.borrow(), cb);
}
#[test]
fn write_sync() {
    let (queue, host, stack, calls) = queue();
    before_sync(&queue, &calls);
    queue.write_sync(WriteData::text("d"), None).unwrap();
    assert_eq!(*stack.borrow(), ["a", "b", "c", "d"].map(WriteData::text));
    assert_eq!(*calls.borrow(), ["a", "b", "c"]);
    after_sync(
        &queue,
        &host,
        &stack,
        &calls,
        &["a", "b", "c", "d"],
        &["a", "b", "c"],
    );
}
#[test]
fn write_sync_reentry_does_not_overflow_callstack() {
    let count = Rc::new(Cell::new(0));
    let output = count.clone();
    let queue = WriteBuffer::new(Host::default(), move |queue, data, _| {
        let WriteData::Text(text) = data else {
            panic!()
        };
        let number = String::from_utf16(text).unwrap().parse::<usize>().unwrap();
        output.set(number);
        if number < 10000 {
            queue.write_sync(WriteData::text(&(number + 1).to_string()), None)?;
        }
        Ok(WriteAction::Complete)
    });
    queue.write_sync(WriteData::text("1"), None).unwrap();
    assert_eq!(count.get(), 10000);
}
#[test]
fn write_sync_max_subsequent_calls() {
    let last = Rc::new(Cell::new(0));
    let output = last.clone();
    let queue = WriteBuffer::new(Host::default(), move |queue, data, _| {
        let WriteData::Text(text) = data else {
            panic!()
        };
        let number = String::from_utf16(text).unwrap().parse::<usize>().unwrap();
        output.set(number);
        if number < 1_000_000 {
            queue.write_sync(WriteData::text(&(number + 1).to_string()), Some(10.0))?;
        }
        Ok(WriteAction::Complete)
    });
    queue.write_sync(WriteData::text("1"), Some(10.0)).unwrap();
    assert_eq!(last.get(), 11);
}
#[test]
fn flush_sync_all_pending() {
    let (queue, host, stack, calls) = queue();
    before_sync(&queue, &calls);
    queue.flush_sync().unwrap();
    assert_eq!(*stack.borrow(), ["a", "b", "c"].map(WriteData::text));
    assert_eq!(*calls.borrow(), ["a", "b", "c"]);
    after_sync(
        &queue,
        &host,
        &stack,
        &calls,
        &["a", "b", "c"],
        &["a", "b", "c"],
    );
}
#[test]
fn flush_sync_no_pending_is_noop() {
    let (queue, _, stack, calls) = queue();
    queue.flush_sync().unwrap();
    assert!(stack.borrow().is_empty());
    assert!(calls.borrow().is_empty());
}
fn parsed(queue: &WriteBuffer<Host>) -> Rc<Cell<usize>> {
    let parsed = Rc::new(Cell::new(0));
    let output = parsed.clone();
    queue
        .on_write_parsed()
        .subscribe(move |_| output.set(output.get() + 1));
    parsed
}
#[test]
fn flush_sync_fires_parsed() {
    let (queue, _, _, _) = queue();
    let parsed = parsed(&queue);
    queue.write(WriteData::text("a"), None).unwrap();
    queue.write(WriteData::text("b"), None).unwrap();
    assert_eq!(parsed.get(), 0);
    queue.flush_sync().unwrap();
    assert_eq!(parsed.get(), 1);
}
#[test]
fn flush_sync_no_pending_does_not_fire_parsed() {
    let (queue, _, _, _) = queue();
    let parsed = parsed(&queue);
    queue.flush_sync().unwrap();
    assert_eq!(parsed.get(), 0);
}
#[test]
fn dispose_cancels_scheduled_inner_write() {
    let (queue, host, stack, _) = queue();
    queue.write(WriteData::text("a"), None).unwrap();
    queue.dispose();
    host.tasks();
    assert!(stack.borrow().is_empty());
    assert_eq!(host.pending_tasks(), 0);
}
#[test]
fn dispose_does_not_fire_parsed_for_pending() {
    let (queue, host, _, _) = queue();
    let parsed = parsed(&queue);
    queue.write(WriteData::text("a"), None).unwrap();
    queue.dispose();
    host.tasks();
    assert_eq!(parsed.get(), 0);
}
#[test]
fn write_after_dispose_is_noop() {
    let (queue, host, stack, _) = queue();
    queue.dispose();
    queue.write(WriteData::text("a"), None).unwrap();
    host.tasks();
    assert!(stack.borrow().is_empty());
}
#[test]
fn dispose_is_idempotent() {
    let (queue, host, stack, _) = queue();
    queue.write(WriteData::text("a"), None).unwrap();
    queue.dispose();
    queue.dispose();
    host.tasks();
    assert!(stack.borrow().is_empty());
}
#[test]
fn async_handler_continuation_skipped_after_dispose() {
    let host = Host::default();
    let queue = WriteBuffer::new(host.clone(), |_, _, _| {
        Ok(WriteAction::Pending(Box::pin(async { Ok(true) })))
    });
    // The original test disposes before its timer runs; retain that exact order.
    queue.write(WriteData::text("a"), None).unwrap();
    queue.dispose();
    host.microtasks();
    host.tasks();
    assert_eq!(queue.pending_units(), 0.0);
}
#[test]
fn handle_user_input_processes_first_chunk_synchronously() {
    let (queue, _, stack, _) = queue();
    queue.handle_user_input();
    queue.write(WriteData::text("a"), None).unwrap();
    assert_eq!(*stack.borrow(), [WriteData::text("a")]);
}
#[test]
fn flush_sync_after_dispose_is_noop() {
    let (queue, host, stack, _) = queue();
    queue.write(WriteData::text("a"), None).unwrap();
    queue.dispose();
    queue.flush_sync().unwrap();
    host.tasks();
    assert!(stack.borrow().is_empty());
}
#[test]
fn write_sync_after_dispose_is_noop() {
    let (queue, _, stack, _) = queue();
    queue.dispose();
    queue.write_sync(WriteData::text("a"), None).unwrap();
    assert!(stack.borrow().is_empty());
}
