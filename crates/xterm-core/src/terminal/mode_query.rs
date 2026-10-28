// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// InputHandler DECRQM reports only the first parameter, unlike set/reset.
use super::{TerminalError, TerminalState};
use crate::parser::params::Params;
use std::rc::Rc;

const NOT_RECOGNIZED: u8 = 0;
const SET: u8 = 1;
const RESET: u8 = 2;
const PERMANENTLY_SET: u8 = 3;
const PERMANENTLY_RESET: u8 = 4;
fn status(enabled: bool) -> u8 {
    if enabled {
        SET
    } else {
        RESET
    }
}

impl TerminalState {
    pub(super) fn request_mode(
        &mut self,
        params: &Params,
        ansi: bool,
        gate: Option<bool>,
        captured_cols: usize,
        captured_modes: Option<crate::core_service::DecPrivateModes>,
    ) -> Result<(), TerminalError> {
        let mode = params.values().first().copied().unwrap_or(0);
        let value = if ansi {
            self.ansi_mode_status(mode)?
        } else {
            let modes =
                captured_modes.unwrap_or_else(|| self.core.dec_private_modes.borrow().clone());
            self.private_mode_status(mode, gate, captured_cols, &modes)?
        };
        self.reply(&format!(
            "\x1b[{}{mode};{value}$y",
            if ansi { "" } else { "?" }
        ))
    }

    fn ansi_mode_status(&self, mode: i32) -> Result<u8, TerminalError> {
        Ok(match mode {
            2 => PERMANENTLY_RESET,
            4 => status(self.core.modes.borrow().insert_mode),
            12 => PERMANENTLY_SET,
            20 => status(self.option_bool("convertEol")?),
            _ => NOT_RECOGNIZED,
        })
    }

    fn private_mode_status(
        &self,
        mode: i32,
        gate: Option<bool>,
        captured_cols: usize,
        modes: &crate::core_service::DecPrivateModes,
    ) -> Result<u8, TerminalError> {
        Ok(match mode {
            1 => status(modes.application_cursor_keys),
            3 if gate == Some(true) => match captured_cols {
                80 => RESET,
                132 => SET,
                _ => NOT_RECOGNIZED,
            },
            6 => status(modes.origin),
            7 => status(modes.wraparound),
            8 => PERMANENTLY_SET,
            9 => status(self.mouse.active_protocol() == "X10"),
            12 => status(self.option_bool("cursorBlink")?),
            25 => status(!self.core.is_cursor_hidden.get()),
            45 => status(modes.reverse_wraparound),
            66 => status(modes.application_keypad),
            67 => PERMANENTLY_RESET,
            1000 => status(self.mouse.active_protocol() == "VT200"),
            1002 => status(self.mouse.active_protocol() == "DRAG"),
            1003 => status(self.mouse.active_protocol() == "ANY"),
            1004 => status(modes.send_focus),
            1005 => PERMANENTLY_RESET,
            1006 => status(self.mouse.active_encoding() == "SGR"),
            1015 => PERMANENTLY_RESET,
            1016 => status(self.mouse.active_encoding() == "SGR_PIXELS"),
            1048 => SET,
            47 | 1047 | 1049 => status(Rc::ptr_eq(&self.buffers.active(), &self.buffers.alt())),
            2004 => status(modes.bracketed_paste_mode),
            2026 => status(modes.synchronized_output),
            9001 if gate == Some(true) => status(modes.win32_input_mode),
            _ => NOT_RECOGNIZED,
        })
    }
}
