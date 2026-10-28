// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use std::{any::Any, cell::RefCell, rc::Rc};
use xterm_core::input::write_buffer::{WriteAction, WriteBuffer, WriteData, WritePayload};
#[derive(Debug)]
struct MutableBytes(Rc<RefCell<Vec<u8>>>);
impl WritePayload for MutableBytes {
    fn len(&self) -> usize {
        self.0.borrow().len()
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
fn external(bytes: &Rc<RefCell<Vec<u8>>>) -> WriteData {
    WriteData::External(Rc::new(MutableBytes(bytes.clone())))
}
fn snapshot(data: &WriteData) -> Vec<u8> {
    match data {
        WriteData::External(payload) => payload
            .as_any()
            .downcast_ref::<MutableBytes>()
            .unwrap()
            .0
            .borrow()
            .clone(),
        WriteData::Bytes(bytes) => bytes.clone(),
        WriteData::Text(_) => panic!("byte-view scenario"),
    }
}
#[test]
fn queued_adapter_view_is_read_at_parse_time_not_enqueue_time() {
    let host = Host::default();
    let log = Rc::new(RefCell::new(vec![]));
    let output = log.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, data, _| {
        output.borrow_mut().push(snapshot(data));
        Ok(WriteAction::Complete)
    });
    let bytes = Rc::new(RefCell::new(vec![65, 66]));
    queue.write(external(&bytes), None).unwrap();
    bytes.borrow_mut()[0] = 67;
    host.tasks();
    assert_eq!(*log.borrow(), [vec![67, 66]]);
}
#[test]
fn detached_view_is_truthy_and_uses_its_post_parse_length() {
    let host = Host::default();
    host.advance(100.0);
    let clock = host.clone();
    let log = Rc::new(RefCell::new(vec![]));
    let output = log.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, data, _| {
        output.borrow_mut().push(snapshot(data));
        clock.advance(12.0);
        Ok(WriteAction::Complete)
    });
    let bytes = Rc::new(RefCell::new(vec![65, 66]));
    queue.write(external(&bytes), None).unwrap();
    queue.write(WriteData::Bytes(vec![88]), None).unwrap();
    bytes.borrow_mut().clear();
    host.task().unwrap();
    assert_eq!(*log.borrow(), [Vec::<u8>::new()]);
    assert_eq!(queue.pending_units(), 3.0);
    assert_eq!(queue.buffer_offset(), 1);
    assert_eq!(queue.buffered_chunks(), 2);
    host.tasks();
    assert_eq!(queue.pending_units(), 0.0);
    assert_eq!(*log.borrow(), [vec![], vec![88]]);
}
#[test]
fn synchronous_empty_adapter_view_is_processed_as_an_object() {
    let host = Host::default();
    let log = Rc::new(RefCell::new(vec![]));
    let output = log.clone();
    let queue = WriteBuffer::new(host, move |_, data, _| {
        output.borrow_mut().push(snapshot(data));
        Ok(WriteAction::Complete)
    });
    let bytes = Rc::new(RefCell::new(vec![]));
    queue.write_sync(external(&bytes), None).unwrap();
    assert_eq!(*log.borrow(), [Vec::<u8>::new()]);
}
