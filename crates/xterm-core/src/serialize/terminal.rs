// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{
    append, scrollback_start, serialize_buffer, BufferAccess, CellRange, Position, SerializeError,
    SerializeOptions,
};
use crate::buffer::AttributeData;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MouseTracking {
    #[default]
    None,
    X10,
    Vt200,
    Drag,
    Any,
}

#[derive(Clone, Debug)]
pub struct SerializeModes {
    pub application_cursor_keys: bool,
    pub application_keypad: bool,
    pub bracketed_paste: bool,
    pub insert: bool,
    pub origin: bool,
    pub reverse_wraparound: bool,
    pub send_focus: bool,
    pub wraparound: bool,
    pub mouse_tracking: MouseTracking,
    pub show_cursor: bool,
}

impl Default for SerializeModes {
    fn default() -> Self {
        Self {
            application_cursor_keys: false,
            application_keypad: false,
            bracketed_paste: false,
            insert: false,
            origin: false,
            reverse_wraparound: false,
            send_focus: false,
            wraparound: true,
            mouse_tracking: MouseTracking::None,
            show_cursor: true,
        }
    }
}

/// All references are borrowed for one synchronous call, so live state is read
/// without retaining pointers or blocking later terminal writes.
pub struct SerializeState<'a> {
    pub normal: &'a dyn BufferAccess,
    pub alternate: &'a dyn BufferAccess,
    pub active_alternate: bool,
    pub cols: usize,
    pub rows: usize,
    pub current_attrs: &'a AttributeData,
    pub modes: &'a SerializeModes,
    /// Margins belong to the active buffer.
    pub scroll_top: usize,
    pub scroll_bottom: usize,
}

pub fn serialize_terminal(
    state: &SerializeState<'_>,
    options: &SerializeOptions,
) -> Result<Vec<u16>, SerializeError> {
    let mut content = if let Some(range) = options.range {
        serialize_buffer(
            state.normal,
            state.cols,
            state.rows,
            state.current_attrs,
            CellRange {
                start: Position {
                    x: 0,
                    y: range.start,
                },
                end: Position {
                    x: state.cols,
                    y: range.end,
                },
            },
            true,
        )?
    } else {
        by_scrollback(state, state.normal, options.scrollback)?
    };
    if !options.exclude_alt_buffer && state.active_alternate {
        append(&mut content, "\x1b[?1049h\x1b[H");
        content.extend(by_scrollback(state, state.alternate, None)?);
    }
    if !options.exclude_modes {
        append_modes(&mut content, state.modes);
        if state.scroll_top != 0 || state.scroll_bottom != state.rows.saturating_sub(1) {
            append(
                &mut content,
                &format!("\x1b[{};{}r", state.scroll_top + 1, state.scroll_bottom + 1),
            );
        }
    }
    Ok(content)
}

fn by_scrollback(
    state: &SerializeState<'_>,
    buffer: &dyn BufferAccess,
    scrollback: Option<isize>,
) -> Result<Vec<u16>, SerializeError> {
    let start = scrollback_start(buffer.len(), state.rows, scrollback);
    if start >= buffer.len() {
        return Err(SerializeError::InvalidRange);
    }
    serialize_buffer(
        buffer,
        state.cols,
        state.rows,
        state.current_attrs,
        CellRange {
            start: Position { x: 0, y: start },
            end: Position {
                x: state.cols,
                y: buffer.len() - 1,
            },
        },
        false,
    )
}

fn append_modes(out: &mut Vec<u16>, m: &SerializeModes) {
    for (enabled, seq) in [
        (m.application_cursor_keys, "\x1b[?1h"),
        (m.application_keypad, "\x1b[?66h"),
        (m.bracketed_paste, "\x1b[?2004h"),
        (m.insert, "\x1b[4h"),
        (m.origin, "\x1b[?6h"),
        (m.reverse_wraparound, "\x1b[?45h"),
        (m.send_focus, "\x1b[?1004h"),
        (!m.wraparound, "\x1b[?7l"),
    ] {
        if enabled {
            append(out, seq);
        }
    }
    append(
        out,
        match m.mouse_tracking {
            MouseTracking::None => "",
            MouseTracking::X10 => "\x1b[?9h",
            MouseTracking::Vt200 => "\x1b[?1000h",
            MouseTracking::Drag => "\x1b[?1002h",
            MouseTracking::Any => "\x1b[?1003h",
        },
    );
    if !m.show_cursor {
        append(out, "\x1b[?25l");
    }
}
