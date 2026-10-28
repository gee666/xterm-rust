// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use std::{any::Any, cell::RefCell, rc::Rc};
use xterm_core::input::write_buffer::{
    WriteAction, WriteBuffer, WriteData, WriteError, WritePayload,
};
#[derive(Debug)]
struct Length {
    error: RefCell<Option<WriteError>>,
}
impl WritePayload for Length {
    fn len(&self) -> Result<usize, WriteError> {
        if let Some(error) = self.error.borrow().clone() {
            Err(error)
        } else {
            Ok(1)
        }
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[test]
fn length_failure_after_scheduling_does_not_accept_chunk() {
    let (queue, host, stack, _) = queue();
    let error = WriteError::external(std::io::Error::other("length getter failed"));
    let payload = Rc::new(Length {
        error: RefCell::new(Some(error.clone())),
    });
    assert_eq!(queue.write(WriteData::External(payload), None), Err(error));
    assert_eq!(queue.pending_units(), 0.0);
    assert_eq!(queue.buffered_chunks(), 0);
    assert!(queue.has_scheduled_write());
    host.tasks();
    assert!(stack.borrow().is_empty());
}
#[test]
fn length_failure_after_callback_keeps_advanced_offset_and_original_error() {
    let host = Host::default();
    let payload = Rc::new(Length {
        error: RefCell::new(None),
    });
    let target = payload.clone();
    let error = WriteError::external(std::io::Error::other("late length getter failed"));
    let failure = error.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, _, _| {
        *target.error.borrow_mut() = Some(failure.clone());
        Ok(WriteAction::Complete)
    });
    queue
        .write(WriteData::External(payload.clone()), None)
        .unwrap();
    assert_eq!(host.task(), Err(error));
    assert_eq!(queue.buffer_offset(), 1);
    assert_eq!(queue.pending_units(), 1.0);
    payload.error.borrow_mut().take();
    queue.dispose();
}
