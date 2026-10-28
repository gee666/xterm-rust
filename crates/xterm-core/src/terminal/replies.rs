// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{TerminalError, TerminalEvent, TerminalState};
use crate::{
    buffer::AttributeData, core_service::CursorStyle, options::OptionValue, parser::params::Params,
};

impl TerminalState {
    pub(crate) fn device_attributes(
        &mut self,
        params: &Params,
        secondary: bool,
    ) -> Result<(), TerminalError> {
        if params.values().first().copied().unwrap_or(0) > 0 {
            return Ok(());
        }
        let term = self.options.get("termName")?.text_utf16()?;
        let term = String::from_utf16_lossy(&term);
        let response = if secondary {
            if term.starts_with("xterm") {
                "\x1b[>0;276;0c"
            } else if term.starts_with("rxvt-unicode") {
                "\x1b[>85;95;0c"
            } else if term.starts_with("screen") {
                "\x1b[>83;40003;0c"
            } else if term.starts_with("linux") {
                "0c"
            } else {
                return Ok(());
            }
        } else if term.starts_with("xterm")
            || term.starts_with("rxvt-unicode")
            || term.starts_with("screen")
        {
            "\x1b[?1;2c"
        } else if term.starts_with("linux") {
            "\x1b[?6c"
        } else {
            return Ok(());
        };
        self.reply(response)
    }
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
        if params.values().first().copied().unwrap_or(0) == 0 {
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
    pub(crate) fn soft_reset(&mut self) -> Result<(), TerminalError> {
        self.core.is_cursor_hidden.set(false);
        self.events.borrow_mut().push(TerminalEvent::SyncScrollBar);
        let active = self.active_buffer();
        let mut b = active.borrow_mut();
        b.scroll_top = 0;
        b.scroll_bottom = self.rows() - 1;
        self.attributes = AttributeData::default();
        self.core.reset();
        self.charset.reset();
        b.saved_x = 0;
        b.saved_y = i64::try_from(b.ybase).map_err(|_| TerminalError::InvalidDimensions)?;
        b.saved_cur_attr_data.fg = 0;
        b.saved_cur_attr_data.bg = 0;
        b.saved_charset = None;
        self.core.dec_private_modes.borrow_mut().origin = false;
        Ok(())
    }
    pub(crate) fn status_string(&mut self, data: &[u16]) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let b = active.borrow();
        let response = match data {
            [34, 113] => format!("P1$r{}\"q", u8::from(self.attributes.is_protected() != 0)),
            [34, 112] => "P1$r61;1\"p".to_owned(),
            [114] => format!("P1$r{};{}r", b.scroll_top + 1, b.scroll_bottom + 1),
            [109] => "P1$r0m".to_owned(),
            [32, 113] => {
                let style = self.options.get("cursorStyle")?;
                let base = if style == OptionValue::from("block") {
                    2
                } else if style == OptionValue::from("underline") {
                    4
                } else if style == OptionValue::from("bar") {
                    6
                } else {
                    0
                };
                format!(
                    "P1$r{} q",
                    base - i32::from(self.option_bool("cursorBlink")?)
                )
            }
            _ => "P0$r".to_owned(),
        };
        drop(b);
        self.reply(&format!("\x1b{response}\x1b\\"))
    }
}
