// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalEvent, TerminalState};
use crate::{core_service::CursorStyle, parser::params::Params};

impl TerminalState {
    pub(crate) fn device_status(
        &mut self,
        params: &Params,
        private: bool,
    ) -> Result<(), TerminalError> {
        match params.values().first().copied().unwrap_or(0) {
            5 if !private => self.reply("\x1b[0n")?,
            6 => {
                let active = self.active_buffer();
                let b = active.borrow();
                let text = format!(
                    "\x1b[{}{};{}R",
                    if private { "?" } else { "" },
                    b.y + 1,
                    b.x + 1
                );
                drop(b);
                self.reply(&text)?;
            }
            996 if private && self.mode_option("vtExtensions", "colorSchemeQuery", true)? => self
                .events
                .borrow_mut()
                .push(TerminalEvent::ColorSchemeQuery),
            _ => {}
        }
        Ok(())
    }
    pub(crate) fn xt_version(&mut self, params: &Params) -> Result<(), TerminalError> {
        if params.values().first().is_none_or(|value| *value <= 0) {
            self.reply(&format!(
                "\x1bP>|xterm.js({})\x1b\\",
                crate::version::XTERM_VERSION
            ))?;
        }
        Ok(())
    }
    pub(crate) fn cursor_style(&mut self, params: &Params) -> Result<(), TerminalError> {
        let value = params.values().first().copied().unwrap_or(1);
        let mut modes = self.core.dec_private_modes.borrow_mut();
        if value == 0 {
            modes.cursor_style = None;
            modes.cursor_blink = None;
        } else {
            match value {
                1 | 2 => modes.cursor_style = Some(CursorStyle::Block),
                3 | 4 => modes.cursor_style = Some(CursorStyle::Underline),
                5 | 6 => modes.cursor_style = Some(CursorStyle::Bar),
                _ => {}
            }
            modes.cursor_blink = Some(value % 2 == 1);
        }
        Ok(())
    }
}
