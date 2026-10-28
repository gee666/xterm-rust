// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use xterm_core::terminal::CoreTerminal;
pub fn lines(t: &CoreTerminal, count: usize) -> Vec<String> {
    let active = t.state().active_buffer();
    let b = active.borrow();
    (0..count)
        .map(|i| {
            String::from_utf16(
                &b.lines
                    .get(b.ybase + i)
                    .unwrap()
                    .borrow()
                    .translate_to_string(true, None, None, None),
            )
            .unwrap()
        })
        .collect()
}
