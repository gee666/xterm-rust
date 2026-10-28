// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// State-only mode mutations. Callback-bearing operations live in mode_callbacks.
use super::{TerminalError, TerminalEvent, TerminalState};
use crate::charset::DEFAULT_CHARSET;

impl TerminalState {
    pub(super) fn mode_option(
        &self,
        group: &str,
        key: &str,
        default: bool,
    ) -> Result<bool, TerminalError> {
        super::commands::mode_callbacks::option_flag(&self.options, group, key, default)
    }

    pub(super) fn apply_state_mode(
        &mut self,
        private: bool,
        mode: i32,
        enabled: bool,
    ) -> Result<(), TerminalError> {
        if !private {
            if mode == 4 {
                self.core.modes.borrow_mut().insert_mode = enabled;
            }
            return Ok(());
        }
        match mode {
            1 => {
                self.core
                    .dec_private_modes
                    .borrow_mut()
                    .application_cursor_keys = enabled
            }
            2 if enabled => {
                for level in 0..4 {
                    self.charset.set_g_charset(level, DEFAULT_CHARSET)?;
                }
            }
            6 => {
                self.core.dec_private_modes.borrow_mut().origin = enabled;
                self.set_cursor(0, 0)?;
            }
            7 => self.core.dec_private_modes.borrow_mut().wraparound = enabled,
            25 => self.core.is_cursor_hidden.set(!enabled),
            45 => self.core.dec_private_modes.borrow_mut().reverse_wraparound = enabled,
            66 => {
                // The CSI adapter logs before borrowing terminal state.
                self.core.dec_private_modes.borrow_mut().application_keypad = enabled;
                self.events.borrow_mut().push(TerminalEvent::SyncScrollBar);
            }
            1004 => {
                self.core.dec_private_modes.borrow_mut().send_focus = enabled;
                if enabled {
                    self.events.borrow_mut().push(TerminalEvent::SendFocus);
                }
            }
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
            2004 => {
                self.core
                    .dec_private_modes
                    .borrow_mut()
                    .bracketed_paste_mode = enabled
            }
            2026 => {
                self.core.dec_private_modes.borrow_mut().synchronized_output = enabled;
                if !enabled {
                    self.events.borrow_mut().push(TerminalEvent::Render {
                        start: 0,
                        end: self.rows() - 1,
                    });
                }
            }
            // Unsupported modes have source-defined no-op state semantics.
            // Their debug messages run in the unborrowed CSI adapter.
            _ => {}
        }
        Ok(())
    }
}
