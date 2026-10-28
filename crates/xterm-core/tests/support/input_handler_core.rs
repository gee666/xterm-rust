// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
use xterm_core::{options::OptionValue, terminal::CoreTerminal};

pub fn make(cols: usize, rows: usize) -> CoreTerminal {
    let mut terminal = CoreTerminal::new(80, 30, 1000).unwrap();
    terminal.resize(cols, rows).unwrap();
    let options = terminal.state().options.clone();
    options.set("scrollback", OptionValue::Number(1.0)).unwrap();
    terminal
}

pub fn cursor(terminal: &CoreTerminal) -> (usize, usize) {
    let active = terminal.state().active_buffer();
    let buffer = active.borrow();
    (buffer.x, buffer.y)
}

pub fn lines(terminal: &CoreTerminal, limit: usize) -> Vec<String> {
    let active = terminal.state().active_buffer();
    let buffer = active.borrow();
    (0..limit)
        .filter_map(|index| buffer.lines.get(index))
        .map(|row| {
            String::from_utf16(&row.borrow().translate_to_string(true, None, None, None)).unwrap()
        })
        .collect()
}
