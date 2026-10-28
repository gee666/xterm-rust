// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Cursor, tab, margin and repeated-grapheme CSI commands from InputHandler.ts.
use super::{TerminalError, TerminalState};
use crate::{parser::params::Params, unicode::UnicodeCharProperties};

pub(super) fn parameter(params: &Params, index: usize, default: i64) -> i64 {
    match params.values().get(index).copied().unwrap_or(0) {
        0 => default,
        value => i64::from(value),
    }
}
pub(super) fn count(params: &Params) -> Result<usize, TerminalError> {
    usize::try_from(parameter(params, 0, 1))
        .map_err(|_| TerminalError::InvalidParameter("negative CSI repeat count"))
}
impl TerminalState {
    pub(crate) fn dispatch_csi(
        &mut self,
        final_byte: &str,
        params: &Params,
        selective: bool,
    ) -> Result<bool, TerminalError> {
        let amount = parameter(params, 0, 1);
        match final_byte {
            "A" | "B" | "E" | "F" => {
                let up = matches!(final_byte, "A" | "F");
                let active = self.active_buffer();
                let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
                let margin = if up {
                    b.y as i64 - b.scroll_top as i64
                } else {
                    b.scroll_bottom as i64 - b.y as i64
                };
                let delta = if margin >= 0 {
                    amount.min(margin)
                } else {
                    amount
                };
                drop(b);
                self.move_csi_cursor(0, if up { -delta } else { delta })?;
                if matches!(final_byte, "E" | "F") {
                    active
                        .try_borrow_mut()
                        .map_err(|_| TerminalError::Borrowed)?
                        .x = 0;
                }
            }
            "C" | "a" => self.move_csi_cursor(amount, 0)?,
            "D" => self.move_csi_cursor(-amount, 0)?,
            "G" | "`" => {
                let y = self
                    .active_buffer()
                    .try_borrow()
                    .map_err(|_| TerminalError::Borrowed)?
                    .y;
                self.set_cursor(amount - 1, y as i64)?;
            }
            "H" | "f" => self.set_cursor(parameter(params, 1, 1) - 1, amount - 1)?,
            "d" => {
                let x = self
                    .active_buffer()
                    .try_borrow()
                    .map_err(|_| TerminalError::Borrowed)?
                    .x;
                self.set_cursor(x as i64, amount - 1)?;
            }
            "e" => self.move_csi_cursor(0, amount)?,
            "I" | "Z" => self.csi_tabs(params, final_byte == "Z")?,
            "g" => {
                let active = self.active_buffer();
                let mut b = active
                    .try_borrow_mut()
                    .map_err(|_| TerminalError::Borrowed)?;
                match parameter(params, 0, 0) {
                    0 => {
                        let x = b.x;
                        b.tabs.remove(&x);
                    }
                    3 => b.tabs.clear(),
                    _ => {}
                }
            }
            "r" => self.csi_margins(params)?,
            "s" => self.save_cursor()?,
            "u" => self.restore_cursor()?,
            _ => return self.dispatch_edits(final_byte, params, selective),
        }
        Ok(true)
    }
    pub(super) fn restrict_csi_cursor(&mut self, max_col: usize) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let mut b = active
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        b.x = b.x.min(max_col);
        b.y = if self.core.dec_private_modes.borrow().origin {
            b.y.clamp(b.scroll_top, b.scroll_bottom)
        } else {
            b.y.min(self.rows() - 1)
        };
        self.dirty_row(b.y);
        Ok(())
    }
    fn move_csi_cursor(&mut self, x: i64, y: i64) -> Result<(), TerminalError> {
        self.restrict_csi_cursor(self.cols() - 1)?;
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let (x, y) = (b.x as i64 + x, b.y as i64 + y);
        drop(b);
        // Source applies origin placement again after restricting the old cursor.
        self.set_cursor(x, y)
    }
    fn csi_tabs(&mut self, params: &Params, backward: bool) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let mut b = active
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        if b.x >= self.cols() {
            return Ok(());
        }
        for _ in 0..count(params)? {
            let next = if backward {
                b.prev_stop(None)
            } else {
                b.next_stop(None)
            };
            if next == b.x {
                break;
            }
            b.x = next;
        }
        Ok(())
    }
    fn csi_margins(&mut self, params: &Params) -> Result<(), TerminalError> {
        let top = parameter(params, 0, 1);
        let bottom = parameter(params, 1, self.rows() as i64);
        let bottom = if bottom > self.rows() as i64 {
            self.rows() as i64
        } else {
            bottom
        };
        if bottom > top {
            if top <= 0 || bottom <= 0 {
                return Err(TerminalError::InvalidParameter(
                    "negative CSI scroll margin",
                ));
            }
            let active = self.active_buffer();
            let mut b = active
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            b.scroll_top = (top - 1) as usize;
            b.scroll_bottom = (bottom - 1) as usize;
            drop(b);
            self.set_cursor(0, 0)?;
        }
        Ok(())
    }
    pub(super) fn repeat_data(&self, params: &Params) -> Result<Option<Vec<u32>>, TerminalError> {
        let preceding = self.preceding_join_state.clone();
        if preceding.get() == 0 {
            return Ok(None);
        }
        let width = usize::from(UnicodeCharProperties::from_raw(preceding.get()).width());
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let row = self.line(b.y)?;
        let text = match b.x.checked_sub(width) {
            Some(x) => row
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .get_string(x)
                .unwrap_or_default(),
            None => Vec::new(),
        };
        drop(b);
        let mut glyph = Vec::new();
        glyph
            .try_reserve(text.len())
            .map_err(|_| TerminalError::AllocationFailed)?;
        let mut i = 0;
        while i < text.len() {
            let first = text[i];
            let code = if (0xd800..=0xdbff).contains(&first)
                && i + 1 < text.len()
                && (0xdc00..=0xdfff).contains(&text[i + 1])
            {
                i += 1;
                (u32::from(first) - 0xd800) * 0x400 + u32::from(text[i]) - 0xdc00 + 0x10000
            } else {
                u32::from(first)
            };
            glyph.push(code);
            i += 1;
        }
        let count = count(params)?;
        let length = glyph
            .len()
            .checked_mul(count)
            .ok_or(TerminalError::AllocationFailed)?;
        let mut data = Vec::new();
        data.try_reserve_exact(length)
            .map_err(|_| TerminalError::AllocationFailed)?;
        for _ in 0..count {
            data.extend_from_slice(&glyph);
        }
        Ok(Some(data))
    }
}
