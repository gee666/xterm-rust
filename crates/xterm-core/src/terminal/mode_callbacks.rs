// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Callback-bearing mode operations run without borrowing terminal state.
use super::super::{TerminalError, TerminalEvent, TerminalState};
use crate::options::{OptionValue, OptionsService};
use std::{cell::RefCell, rc::Rc};

type Shared = Rc<RefCell<TerminalState>>;
fn options(shared: &Shared) -> Result<Rc<OptionsService>, TerminalError> {
    Ok(Rc::clone(
        &shared
            .try_borrow()
            .map_err(|_| TerminalError::Borrowed)?
            .options,
    ))
}

pub(crate) fn option_flag(
    options: &OptionsService,
    group: &str,
    key: &str,
    default: bool,
) -> Result<bool, TerminalError> {
    let bag = options.get(group)?;
    let value = match bag {
        OptionValue::Null | OptionValue::Undefined if group != "windowOptions" => {
            OptionValue::Undefined
        }
        value => value.get_property(key)?,
    };
    Ok(match value {
        OptionValue::Null | OptionValue::Undefined => default,
        value => value.is_truthy(),
    })
}

pub(super) fn apply(
    shared: &Shared,
    private: bool,
    mode: i32,
    enabled: bool,
) -> Result<bool, TerminalError> {
    if !private {
        if mode != 20 {
            return Ok(false);
        }
        options(shared)?.set("convertEol", OptionValue::Bool(enabled))?;
        return Ok(true);
    }
    match mode {
        3 => {
            if option_flag(
                options(shared)?.as_ref(),
                "windowOptions",
                "setWinLines",
                false,
            )? {
                let service = Rc::clone(
                    &shared
                        .try_borrow()
                        .map_err(|_| TerminalError::Borrowed)?
                        .buffer_service,
                );
                service.resize(if enabled { 132 } else { 80 }, service.rows())?;
                // Resize observers run before the source reset request.
                super::super::dispatch_events(shared)?;
                super::super::request_reset_state(shared)?;
            }
        }
        12 => {
            let options = options(shared)?;
            if option_flag(&options, "quirks", "allowSetCursorBlink", false)? {
                options.set("cursorBlink", OptionValue::Bool(enabled))?;
            }
        }
        9 | 1000 | 1002 | 1003 => {
            let mouse = Rc::clone(
                &shared
                    .try_borrow()
                    .map_err(|_| TerminalError::Borrowed)?
                    .mouse,
            );
            let protocol = if !enabled {
                "NONE"
            } else {
                match mode {
                    9 => "X10",
                    1000 => "VT200",
                    1002 => "DRAG",
                    _ => "ANY",
                }
            };
            mouse.set_active_protocol(protocol)?;
        }
        47 | 1047 | 1049 => switch_buffer(shared, mode, enabled)?,
        2031 | 9001 => {
            let (key, default) = if mode == 2031 {
                ("colorSchemeQuery", true)
            } else {
                ("win32InputMode", false)
            };
            if option_flag(options(shared)?.as_ref(), "vtExtensions", key, default)? {
                let core = Rc::clone(
                    &shared
                        .try_borrow()
                        .map_err(|_| TerminalError::Borrowed)?
                        .core,
                );
                let mut modes = core.dec_private_modes.borrow_mut();
                if mode == 2031 {
                    modes.color_scheme_updates = enabled;
                } else {
                    modes.win32_input_mode = enabled;
                }
            }
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn switch_buffer(shared: &Shared, mode: i32, enabled: bool) -> Result<(), TerminalError> {
    if enabled && mode == 1049 {
        shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?
            .save_cursor()?;
    }
    if option_flag(
        options(shared)?.as_ref(),
        "vtExtensions",
        "kittyKeyboard",
        false,
    )? {
        let core = Rc::clone(
            &shared
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .core,
        );
        let mut kitty = core.kitty_keyboard.borrow_mut();
        if enabled {
            kitty.main_flags = kitty.flags;
            kitty.flags = kitty.alt_flags;
        } else {
            kitty.alt_flags = kitty.flags;
            kitty.flags = kitty.main_flags;
        }
    }
    let (service, attributes) = {
        let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        (Rc::clone(&state.buffer_service), state.erase_attributes())
    };
    let previous = service.buffers.active();
    if enabled {
        service.buffers.activate_alt_buffer(Some(&attributes))?;
    } else {
        service.buffers.activate_normal_buffer()?;
    }
    let active = service.buffers.active();
    if !Rc::ptr_eq(&previous, &active) {
        let (top, bottom) = {
            let buffer = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
            (buffer.scroll_top, buffer.scroll_bottom)
        };
        let mut state = shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        state.dirty_row(top);
        state.dirty_row(bottom);
    }
    // Scroll and buffer-activation observers may reset or modify terminal state.
    super::super::dispatch_events(shared)?;
    let mut state = shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    if !enabled && mode == 1049 {
        state.restore_cursor()?;
    }
    state.core.is_cursor_initialized.set(true);
    state.events.borrow_mut().push(TerminalEvent::Render {
        start: 0,
        end: state.rows() - 1,
    });
    state.events.borrow_mut().push(TerminalEvent::SyncScrollBar);
    Ok(())
}
