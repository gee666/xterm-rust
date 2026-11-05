// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::rc::Rc;
use xterm_core::{buffer::BufferService, headless::Terminal};

fn snapshot(buffers: &BufferService, tag: &str) -> String {
    let mut fields = vec![tag.to_owned()];
    for buffer in [buffers.buffers.normal(), buffers.buffers.alt()] {
        let buffer = buffer.borrow();
        let lengths = (0..buffer.lines.len())
            .map(|index| buffer.lines.get(index).unwrap().borrow().len().to_string())
            .collect::<Vec<_>>()
            .join(",");
        fields.push(lengths);
        fields.push(buffer.has_pending_cleanup().to_string());
    }
    fields.join("|")
}

#[test]
fn source_alternate_resize_cleanup_traces() {
    // Source captures both idle and priority timer hosts. Native buffer state is
    // host-independent; the real browser timer path remains wasm_buffer_cleanup.
    for case in include_str!("fixtures/buffer-cleanup.tsv").lines() {
        let fields: Vec<_> = case.split('\t').collect();
        let cols: usize = fields[1].parse().unwrap();
        let scrollback = fields[2].parse().unwrap();
        let terminal = Terminal::new(cols, 4, scrollback).unwrap();
        terminal.write("normal\x1b[?1049halt").unwrap();
        let buffers = terminal
            .with_state(|state| Rc::clone(&state.buffer_service))
            .unwrap();
        let mut actual = vec![snapshot(&buffers, "initial")];
        terminal.resize(cols / 4, 4).unwrap();
        actual.push(snapshot(&buffers, "shrink"));
        terminal.resize(cols / 4 + 1, 4).unwrap();
        actual.push(snapshot(&buffers, "grow"));
        for buffer in [buffers.buffers.normal(), buffers.buffers.alt()] {
            assert!(buffer.borrow_mut().poll_cleanup().unwrap());
            assert!(!buffer.borrow_mut().poll_cleanup().unwrap());
        }
        actual.push(snapshot(&buffers, "cleaned"));
        assert_eq!(actual.join("^"), fields[3], "{case}");
    }
}
