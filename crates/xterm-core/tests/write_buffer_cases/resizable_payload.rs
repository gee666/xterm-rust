// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::write_buffer_support::*;
use std::{any::Any, cell::RefCell, rc::Rc};
use xterm_core::input::write_buffer::{WriteAction, WriteBuffer, WriteData, WritePayload};
#[derive(Debug)]
struct Resizable(Rc<RefCell<Vec<u8>>>);
impl WritePayload for Resizable {
    fn len(&self) -> Result<usize, xterm_core::input::write_buffer::WriteError> {
        Ok(self.0.borrow().len())
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
#[test]
fn resized_view_uses_current_length_even_when_pending_becomes_negative() {
    let host = Host::default();
    host.advance(100.0);
    let clock = host.clone();
    let log = Rc::new(RefCell::new(vec![]));
    let output = log.clone();
    let queue = WriteBuffer::new(host.clone(), move |_, data, _| {
        let data = match data {
            WriteData::External(payload) => payload
                .as_any()
                .downcast_ref::<Resizable>()
                .unwrap()
                .0
                .borrow()
                .clone(),
            WriteData::Bytes(bytes) => bytes.clone(),
            WriteData::Text(_) => panic!("byte scenario"),
        };
        output.borrow_mut().push(data);
        clock.advance(12.0);
        Ok(WriteAction::Complete)
    });
    let bytes = Rc::new(RefCell::new(vec![65, 66]));
    queue
        .write(WriteData::External(Rc::new(Resizable(bytes.clone()))), None)
        .unwrap();
    queue.write(WriteData::Bytes(vec![88]), None).unwrap();
    bytes.borrow_mut().extend([67, 68]);
    host.task().unwrap();
    assert_eq!(*log.borrow(), [vec![65, 66, 67, 68]]);
    assert_eq!(queue.pending_units(), -1.0);
    assert_eq!(queue.buffer_offset(), 1);
    assert_eq!(queue.buffered_chunks(), 2);
    host.tasks();
    assert_eq!(queue.pending_units(), 0.0);
}
