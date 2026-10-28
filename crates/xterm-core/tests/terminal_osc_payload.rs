// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
use xterm_core::{parser::osc::PAYLOAD_LIMIT, terminal::CoreTerminal};

#[test]
fn builtin_title_handler_enforces_the_real_utf16_payload_limit() {
    let mut terminal = CoreTerminal::new(4, 3, 2).unwrap();
    let chunk = vec![u16::from(b'A'); 4096];
    terminal.write("\x1b]2;").unwrap();
    for _ in 0..PAYLOAD_LIMIT / chunk.len() {
        terminal.write_utf16(&chunk).unwrap();
    }
    terminal
        .write_utf16(&chunk[..PAYLOAD_LIMIT % chunk.len()])
        .unwrap();
    terminal.write("\x07").unwrap();
    assert_eq!(terminal.state().title.len(), PAYLOAD_LIMIT);
    assert_eq!(terminal.state().title[0], u16::from(b'A'));
    terminal.drain_events();
    terminal.write("\x1b]2;").unwrap();
    let other = vec![u16::from(b'B'); 4096];
    for _ in 0..PAYLOAD_LIMIT / other.len() {
        terminal.write_utf16(&other).unwrap();
    }
    terminal
        .write_utf16(&other[..PAYLOAD_LIMIT % other.len() + 1])
        .unwrap();
    terminal.write("\x07").unwrap();
    assert_eq!(terminal.state().title.len(), PAYLOAD_LIMIT);
    assert_eq!(terminal.state().title[0], u16::from(b'A'));
}
