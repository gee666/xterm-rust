// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// Buffer.test.ts checks backing-array bytes, not just logical line lengths.
use crate::{buffer::Buffer, options::OptionsService};
use std::rc::Rc;

#[test]
fn original_buffer_idle_cleanup_preserves_then_realigns_exact_backing_bytes() {
    let mut buffer = Buffer::new(true, 80, 24, Rc::new(OptionsService::default())).unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    buffer.resize(39, 24).unwrap();
    for y in 0..24 {
        let line = buffer.lines.get(y).unwrap();
        let line = line.borrow();
        // Vec length models the JS ArrayBuffer; logical length models the view.
        assert_eq!(
            line.data.len() * std::mem::size_of::<[u32; 3]>(),
            80 * 3 * 4
        );
        assert_eq!(line.length * 3, 39 * 3);
        assert_eq!(line.len(), 39);
    }
    assert!(buffer.poll_cleanup().unwrap());
    assert!(!buffer.poll_cleanup().unwrap());
    for y in 0..24 {
        let line = buffer.lines.get(y).unwrap();
        let line = line.borrow();
        assert_eq!(
            line.data.len() * std::mem::size_of::<[u32; 3]>(),
            39 * 3 * 4
        );
    }
}
