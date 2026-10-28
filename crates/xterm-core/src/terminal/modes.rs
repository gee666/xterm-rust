// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Port of InputHandler standard/private mode setters and resets.
use super::{TerminalError, TerminalEvent, TerminalState};
use crate::{charset::DEFAULT_CHARSET, options::OptionValue, parser::params::Params};

impl TerminalState {
    pub(crate) fn dispatch_mode(
        &mut self,
        prefix: Option<&str>,
        final_character: &str,
        params: &Params,
    ) -> Result<bool, TerminalError> {
        match (prefix, final_character) {
            (None, "h" | "l") => self.standard_mode(params, final_character == "h")?,
            (Some("?"), "h" | "l") => self.private_modes(params, final_character == "h")?,
            (None, "$p") => self.request_mode(params, true)?,
            (Some("?"), "$p") => self.request_mode(params, false)?,
            (Some(operation @ ("=" | "?" | ">" | "<")), "u") => {
                self.kitty_mode(operation, params)?;
            }
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn standard_mode(&mut self, params: &Params, enabled: bool) -> Result<(), TerminalError> {
        for value in params.values() {
            match value {
                4 => self.core.modes.borrow_mut().insert_mode = enabled,
                20 => self.options.set("convertEol", OptionValue::Bool(enabled))?,
                _ => {}
            }
        }
        Ok(())
    }

    pub(super) fn mode_option(
        &self,
        group: &str,
        key: &str,
        default: bool,
    ) -> Result<bool, TerminalError> {
        let bag = self.options.get(group)?;
        let value = match bag {
            // vtExtensions and quirks use optional chaining in InputHandler.
            OptionValue::Null | OptionValue::Undefined if group != "windowOptions" => {
                OptionValue::Undefined
            }
            value => value.get_property(key)?,
        };
        Ok(match value {
            OptionValue::Undefined | OptionValue::Null => default,
            value => value.is_truthy(),
        })
    }

    fn private_modes(&mut self, params: &Params, enabled: bool) -> Result<(), TerminalError> {
        for &mode in params.values() {
            self.private_mode(mode, enabled)?;
        }
        Ok(())
    }

    fn private_mode(&mut self, mode: i32, enabled: bool) -> Result<(), TerminalError> {
        match mode {
            1 => {
                self.core
                    .dec_private_modes
                    .borrow_mut()
                    .application_cursor_keys = enabled
            }
            2 if enabled => {
                for level in 0..4 {
                    self.charset
                        .set_g_charset(level, DEFAULT_CHARSET)
                        .map_err(|error| TerminalError::Buffer(error.to_string()))?;
                }
            }
            3 if self.mode_option("windowOptions", "setWinLines", false)? => {
                self.buffer_service
                    .resize(if enabled { 132 } else { 80 }, self.rows())
                    .map_err(|error| TerminalError::Buffer(error.to_string()))?;
                self.events.borrow_mut().push(TerminalEvent::RequestReset);
            }
            6 => {
                self.core.dec_private_modes.borrow_mut().origin = enabled;
                self.set_cursor(0, 0)?;
            }
            7 => self.core.dec_private_modes.borrow_mut().wraparound = enabled,
            12 if self.mode_option("quirks", "allowSetCursorBlink", false)? => {
                self.options
                    .set("cursorBlink", OptionValue::Bool(enabled))?;
            }
            25 => self.core.is_cursor_hidden.set(!enabled),
            45 => self.core.dec_private_modes.borrow_mut().reverse_wraparound = enabled,
            66 => {
                self.mode_debug(if enabled {
                    "Serial port requested application keypad."
                } else {
                    "Switching back to normal keypad."
                })?;
                self.core.dec_private_modes.borrow_mut().application_keypad = enabled;
                self.events.borrow_mut().push(TerminalEvent::SyncScrollBar);
            }
            9 | 1000 | 1002 | 1003 => {
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
                self.mouse.set_active_protocol(protocol)?;
            }
            1004 => {
                self.core.dec_private_modes.borrow_mut().send_focus = enabled;
                if enabled {
                    self.events.borrow_mut().push(TerminalEvent::SendFocus);
                }
            }
            1005 | 1015 => self.mode_debug(&format!(
                "{} {mode} not supported (see #2507)",
                if enabled { "DECSET" } else { "DECRST" }
            ))?,
            1006 | 1016 => {
                let encoding = if !enabled {
                    "DEFAULT"
                } else if mode == 1006 {
                    "SGR"
                } else {
                    "SGR_PIXELS"
                };
                self.mouse.set_active_encoding(encoding)?;
            }
            1048 => {
                if enabled {
                    self.save_cursor()?;
                } else {
                    self.restore_cursor()?;
                }
            }
            47 | 1047 | 1049 => self.switch_mode_buffer(mode, enabled)?,
            2004 => {
                self.core
                    .dec_private_modes
                    .borrow_mut()
                    .bracketed_paste_mode = enabled
            }
            2026 => {
                self.core.dec_private_modes.borrow_mut().synchronized_output = enabled;
                if !enabled {
                    self.dirty_all();
                }
            }
            2031 if self.mode_option("vtExtensions", "colorSchemeQuery", true)? => {
                self.core
                    .dec_private_modes
                    .borrow_mut()
                    .color_scheme_updates = enabled;
            }
            9001 if self.mode_option("vtExtensions", "win32InputMode", false)? => {
                self.core.dec_private_modes.borrow_mut().win32_input_mode = enabled;
            }
            _ => {}
        }
        Ok(())
    }

    fn switch_mode_buffer(&mut self, mode: i32, enabled: bool) -> Result<(), TerminalError> {
        if enabled && mode == 1049 {
            self.save_cursor()?;
        }
        if self.mode_option("vtExtensions", "kittyKeyboard", false)? {
            let mut kitty = self.core.kitty_keyboard.borrow_mut();
            if enabled {
                kitty.main_flags = kitty.flags;
                kitty.flags = kitty.alt_flags;
            } else {
                kitty.alt_flags = kitty.flags;
                kitty.flags = kitty.main_flags;
            }
        }
        if enabled {
            self.buffers
                .activate_alt_buffer(Some(&self.erase_attributes()))
                .map_err(|error| TerminalError::Buffer(error.to_string()))?;
        } else {
            self.buffers
                .activate_normal_buffer()
                .map_err(|error| TerminalError::Buffer(error.to_string()))?;
            if mode == 1049 {
                self.restore_cursor()?;
            }
        }
        self.core.is_cursor_initialized.set(true);
        self.dirty_all();
        self.events.borrow_mut().push(TerminalEvent::SyncScrollBar);
        Ok(())
    }
}
