// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Reply and reset phases that may invoke option conversion or host callbacks.
use super::{TerminalError, TerminalEvent, TerminalState};
use crate::{buffer::AttributeData, options::OptionValue, parser::params::Params};
use std::{cell::RefCell, rc::Rc};

pub(super) fn device_attributes(
    shared: &Rc<RefCell<TerminalState>>,
    params: &Params,
    secondary: bool,
) -> Result<(), TerminalError> {
    let first = params.values().first().copied();
    if first.is_some_and(|value| value > 0) {
        return Ok(());
    }
    let options = Rc::clone(
        &shared
            .try_borrow()
            .map_err(|_| TerminalError::Borrowed)?
            .options,
    );
    let candidates: &[(&str, &str)] = if secondary {
        &[
            ("xterm", "\x1b[>0;276;0c"),
            ("rxvt-unicode", "\x1b[>85;95;0c"),
            ("linux", ""),
            ("screen", "\x1b[>83;40003;0c"),
        ]
    } else {
        &[
            ("xterm", "\x1b[?1;2c"),
            ("rxvt-unicode", "\x1b[?1;2c"),
            ("screen", "\x1b[?1;2c"),
            ("linux", "\x1b[?6c"),
        ]
    };
    for &(prefix, response) in candidates {
        // The source repeats +'' for each short-circuit branch. A conversion may
        // synchronously change options or reset the terminal between attempts.
        let name = options.get("termName")?.concat_utf16()?;
        if name.len() >= prefix.len()
            && name
                .iter()
                .zip(prefix.bytes())
                .all(|(&unit, byte)| unit == u16::from(byte))
        {
            let response = if secondary && prefix == "linux" {
                first.map_or_else(|| "undefinedc".to_owned(), |value| format!("{value}c"))
            } else {
                response.to_owned()
            };
            shared
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .reply(&response)?;
            break;
        }
    }
    Ok(())
}

pub(super) fn soft_reset(shared: &Rc<RefCell<TerminalState>>) -> Result<(), TerminalError> {
    {
        let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        state.core.is_cursor_hidden.set(false);
        state.events.borrow_mut().push(TerminalEvent::SyncScrollBar);
    }
    super::dispatch_events(shared)?;
    let mut state = shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    let active = state.active_buffer();
    let mut b = active
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    b.scroll_top = 0;
    b.scroll_bottom = state.rows() - 1;
    state.replace_attributes(AttributeData::default());
    state.core.reset();
    state.charset.reset();
    b.saved_x = 0;
    b.saved_y = i64::try_from(b.ybase).map_err(|_| TerminalError::InvalidDimensions)?;
    b.saved_cur_attr_data.fg = 0;
    b.saved_cur_attr_data.bg = 0;
    b.saved_charset = None;
    state.core.dec_private_modes.borrow_mut().origin = false;
    Ok(())
}

pub(super) fn status_string(
    shared: &Rc<RefCell<TerminalState>>,
    data: &[u16],
) -> Result<(), TerminalError> {
    let response = match data {
        [34, 113] => {
            let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
            format!("P1$r{}\"q", u8::from(state.attributes.is_protected() != 0))
        }
        [34, 112] => "P1$r61;1\"p".to_owned(),
        [114] => {
            let active = shared
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .active_buffer();
            let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
            format!("P1$r{};{}r", b.scroll_top + 1, b.scroll_bottom + 1)
        }
        [109] => "P1$r0m".to_owned(),
        [32, 113] => {
            let options = Rc::clone(
                &shared
                    .try_borrow()
                    .map_err(|_| TerminalError::Borrowed)?
                    .options,
            );
            let style = OptionValue::String(options.get("cursorStyle")?.to_utf16()?);
            let base = if style == OptionValue::from("block") {
                Some(2)
            } else if style == OptionValue::from("underline") {
                Some(4)
            } else if style == OptionValue::from("bar") {
                Some(6)
            } else {
                None
            };
            let blink = i32::from(options.get("cursorBlink")?.is_truthy());
            let value = base.map_or_else(|| "NaN".to_owned(), |base| (base - blink).to_string());
            format!("P1$r{value} q")
        }
        _ => "P0$r".to_owned(),
    };
    shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?
        .reply(&format!("\x1b{response}\x1b\\"))
}
