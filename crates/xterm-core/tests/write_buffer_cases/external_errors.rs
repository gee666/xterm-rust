// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use std::{cell::Cell, fmt, rc::Rc};
use xterm_core::input::write_buffer::{WriteAction, WriteBuffer, WriteData, WriteError};
#[derive(Debug)]
struct OriginalError(Rc<Cell<usize>>);
impl fmt::Display for OriginalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("original error")
    }
}
impl std::error::Error for OriginalError {}
#[test]
fn actual_error_object_is_retained_after_rejected_handler_and_disposal() {
    let marker = Rc::new(Cell::new(7));
    let original = WriteError::external(OriginalError(marker.clone()));
    let error = original.clone();
    let host = Host::default();
    let queue = WriteBuffer::new(host.clone(), move |_, _, _| {
        let error = error.clone();
        Ok(WriteAction::Pending(Box::pin(async move { Err(error) })))
    });
    queue.write(WriteData::text("a"), None).unwrap();
    host.task().unwrap();
    queue.dispose();
    host.microtasks();
    let errors = host.0.errors.borrow();
    assert_eq!(errors.as_slice(), [original]);
    let typed = errors[0].external_ref::<OriginalError>().unwrap();
    assert!(Rc::ptr_eq(&typed.0, &marker));
    typed.0.set(19);
    assert_eq!(marker.get(), 19);
    assert_eq!(queue.pending_units(), 0.0);
}
