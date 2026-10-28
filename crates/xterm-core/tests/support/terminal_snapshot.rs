// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use std::fmt::Write;
use xterm_core::{
    buffer::Buffer,
    terminal::{CoreTerminal, TerminalEvent},
};
pub fn hex(units: &[u16]) -> String {
    units
        .iter()
        .map(|u| format!("{u:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
pub fn parse_hex(text: &str) -> Vec<u16> {
    if text.is_empty() {
        Vec::new()
    } else {
        text.split(',')
            .map(|s| u16::from_str_radix(s, 16).unwrap())
            .collect()
    }
}
fn buffer(b: &Buffer) -> String {
    let mut result = format!(
        "{},{},{},{},{},{}/",
        b.x, b.y, b.ybase, b.ydisp, b.scroll_top, b.scroll_bottom
    );
    for y in 0..b.lines.len() {
        if y != 0 {
            result.push('/');
        }
        let row = b.lines.get(y).unwrap();
        let row = row.borrow();
        write!(result, "{}:", u8::from(row.is_wrapped)).unwrap();
        for x in 0..row.len() {
            if x != 0 {
                result.push(';');
            }
            let c = row.get_cell(x).unwrap();
            let e = &c.extended;
            write!(
                result,
                "{},{},{},{},{},{}",
                c.content,
                c.fg,
                c.bg,
                hex(&c.get_chars()),
                e.ext(),
                e.url_id()
            )
            .unwrap();
        }
    }
    result
}
pub fn snapshot(t: &CoreTerminal) -> String {
    snapshot_state(&t.state())
}
pub fn snapshot_state(s: &xterm_core::terminal::TerminalState) -> String {
    let m = s.core.dec_private_modes.borrow();
    let a = &s.attributes;
    let normal = s.buffers.normal();
    let alt = s.buffers.alt();
    format!(
        "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}|{}|{}",
        s.cols(),
        s.rows(),
        u8::from(std::rc::Rc::ptr_eq(&s.active_buffer(), &alt)),
        a.fg,
        a.bg,
        a.extended.ext(),
        a.extended.url_id(),
        u8::from(s.core.modes.borrow().insert_mode),
        u8::from(m.origin),
        u8::from(m.wraparound),
        u8::from(m.reverse_wraparound),
        u8::from(m.application_cursor_keys),
        u8::from(m.application_keypad),
        u8::from(m.bracketed_paste_mode),
        u8::from(m.send_focus),
        u8::from(m.synchronized_output),
        buffer(&normal.borrow()),
        buffer(&alt.borrow())
    )
}
pub fn events(t: &mut CoreTerminal) -> String {
    t.drain_events()
        .into_iter()
        .filter_map(|e| event(&e))
        .collect::<Vec<_>>()
        .join("/")
}
pub fn event(e: &TerminalEvent) -> Option<String> {
    match e {
        TerminalEvent::Bell => Some("B".to_owned()),
        TerminalEvent::CursorMove => Some("C".to_owned()),
        TerminalEvent::LineFeed => Some("L".to_owned()),
        TerminalEvent::Scroll(y) => Some(format!("S{y}")),
        TerminalEvent::Data(text) => Some(format!("D{}", hex(text))),
        TerminalEvent::TitleChange(text) => Some(format!("T{}", hex(text))),
        TerminalEvent::Render { start, end } => Some(format!("R{start}:{end}")),
        _ => None,
    }
}
