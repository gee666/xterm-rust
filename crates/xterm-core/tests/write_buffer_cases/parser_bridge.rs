// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use std::{cell::Cell, rc::Rc};
use xterm_core::{
    input::write_buffer::{WriteAction, WriteBuffer, WriteData},
    parser::handler::{DispatchError, HandlerResult},
};
#[test]
fn immediate_parser_failure_is_not_treated_as_complete() {
    let error = match WriteAction::from_parser(Some(HandlerResult::Failed(
        DispatchError::ActiveHandlerRemoved,
    ))) {
        Err(error) => error,
        Ok(_) => panic!("immediate parser failure swallowed"),
    };
    assert_eq!(
        error.external_ref::<DispatchError>(),
        Some(&DispatchError::ActiveHandlerRemoved)
    );
    assert!(matches!(
        WriteAction::from_parser(None),
        Ok(WriteAction::Complete)
    ));
    for value in [true, false] {
        assert!(matches!(
            WriteAction::from_parser(Some(HandlerResult::Ready(value))),
            Ok(WriteAction::Complete)
        ));
    }
}
#[test]
fn fallible_parser_future_reports_error_then_resumes_false() {
    let host = Host::default();
    let count = Rc::new(Cell::new(0));
    let calls = count.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, _, result| {
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            WriteAction::from_parser(Some(HandlerResult::PendingFallible(Box::pin(async {
                Err(DispatchError::ActiveHandlerRemoved)
            }))))
        } else {
            assert_eq!(result, Some(false));
            Ok(WriteAction::Complete)
        }
    });
    queue.write(WriteData::text("a"), None).unwrap();
    host.task().unwrap();
    host.microtasks();
    assert_eq!(count.get(), 1);
    assert_eq!(queue.pending_units(), 1.0);
    assert_eq!(
        host.0.errors.borrow()[0].external_ref::<DispatchError>(),
        Some(&DispatchError::ActiveHandlerRemoved)
    );
    host.microtasks();
    assert_eq!(count.get(), 2);
    assert_eq!(queue.pending_units(), 0.0);
}
#[test]
fn ordinary_parser_future_retains_its_result() {
    let host = Host::default();
    let count = Rc::new(Cell::new(0));
    let calls = count.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, _, result| {
        calls.set(calls.get() + 1);
        if calls.get() == 1 {
            WriteAction::from_parser(Some(HandlerResult::Pending(Box::pin(async { false }))))
        } else {
            assert_eq!(result, Some(false));
            Ok(WriteAction::Complete)
        }
    });
    queue.write(WriteData::text("a"), None).unwrap();
    host.task().unwrap();
    host.microtasks();
    assert_eq!(count.get(), 2);
    assert!(host.0.errors.borrow().is_empty());
    assert_eq!(queue.pending_units(), 0.0);
}
