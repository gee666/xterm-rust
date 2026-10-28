// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use super::{CoreTerminal, TerminalError, TerminalEvent, TerminalState};
use crate::buffer::AttributeData;

impl CoreTerminal {
    pub fn resize(&mut self, cols: usize, rows: usize) -> Result<(), TerminalError> {
        self.ensure_live()?;
        if self.awaiting {
            return Err(TerminalError::Pending);
        }
        if cols > i32::MAX as usize || rows > i32::MAX as usize {
            return Err(TerminalError::InvalidDimensions);
        }
        let mut state = self.state.borrow_mut();
        state
            .buffer_service
            .resize(cols.max(2), rows.max(1))
            .map_err(|e| TerminalError::Buffer(e.to_string()))?;
        state.dirty_all();
        Ok(())
    }
    pub fn scroll_lines(&mut self, lines: i64) -> Result<(), TerminalError> {
        self.ensure_live()?;
        let mut state = self.state.borrow_mut();
        state
            .buffer_service
            .scroll_lines(lines, false)
            .map_err(|e| TerminalError::Buffer(e.to_string()))?;
        state.dirty_all();
        Ok(())
    }
    pub fn scroll_pages(&mut self, pages: i64) -> Result<(), TerminalError> {
        let lines = pages.saturating_mul((self.rows() - 1) as i64);
        self.scroll_lines(lines)
    }
    pub fn scroll_to_top(&mut self) -> Result<(), TerminalError> {
        self.scroll_to_line(0)
    }
    pub fn scroll_to_bottom(&mut self) -> Result<(), TerminalError> {
        let ybase = self.state.borrow().active_buffer().borrow().ybase;
        self.scroll_to_line(ybase)
    }
    pub fn scroll_to_line(&mut self, line: usize) -> Result<(), TerminalError> {
        let ydisp = self.state.borrow().active_buffer().borrow().ydisp;
        let difference = line as i128 - ydisp as i128;
        self.scroll_lines(difference.clamp(i128::from(i64::MIN), i128::from(i64::MAX)) as i64)
    }
    pub fn reset(&mut self) -> Result<(), TerminalError> {
        self.ensure_live()?;
        // Public reset is out-of-band and deliberately retains parser/decoder state.
        self.state.borrow_mut().reset_state()
    }
    pub fn clear(&mut self) -> Result<(), TerminalError> {
        self.ensure_live()?;
        let mut state = self.state.borrow_mut();
        let active = state.active_buffer();
        let mut b = active.borrow_mut();
        b.clear_all_markers()
            .map_err(|e| TerminalError::Buffer(e.to_string()))?;
        let row = b
            .lines
            .get(b.ybase + b.y)
            .ok_or(TerminalError::MissingLine)?;
        b.lines
            .set(0, Some(row))
            .map_err(|e| TerminalError::Buffer(e.to_string()))?;
        b.lines.set_length(1);
        b.ydisp = 0;
        b.ybase = 0;
        b.y = 0;
        for _ in 1..state.rows() {
            b.lines
                .push(b.get_blank_line(&AttributeData::default(), false))
                .map_err(|e| TerminalError::Buffer(e.to_string()))?;
        }
        state.events.borrow_mut().push(TerminalEvent::Scroll(0));
        state.dirty_all();
        Ok(())
    }
}
impl TerminalState {
    pub(crate) fn set_cursor(&mut self, x: i64, y: i64) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let mut b = active.borrow_mut();
        self.dirty_row(b.y);
        let origin = self.core.dec_private_modes.borrow().origin;
        b.x = x.clamp(0, (self.cols() - 1) as i64) as usize;
        let (min, max, y) = if origin {
            (
                b.scroll_top,
                b.scroll_bottom,
                y.saturating_add(b.scroll_top as i64),
            )
        } else {
            (0, self.rows() - 1, y)
        };
        b.y = y.clamp(min as i64, max as i64) as usize;
        self.dirty_row(b.y);
        Ok(())
    }
    pub(crate) fn save_cursor(&mut self) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let mut b = active.borrow_mut();
        b.saved_x = b.x;
        b.saved_y = i64::try_from(
            b.ybase
                .checked_add(b.y)
                .ok_or(TerminalError::InvalidDimensions)?,
        )
        .map_err(|_| TerminalError::InvalidDimensions)?;
        b.saved_cur_attr_data.fg = self.attributes.fg;
        b.saved_cur_attr_data.bg = self.attributes.bg;
        b.saved_charset = self.charset.charset.clone();
        b.saved_charsets = self.charset.charsets().to_vec();
        b.saved_glevel = self.charset.glevel;
        let modes = self.core.dec_private_modes.borrow();
        b.saved_origin_mode = modes.origin;
        b.saved_wraparound_mode = modes.wraparound;
        Ok(())
    }
    pub(crate) fn restore_cursor(&mut self) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let b = active.borrow();
        let x = b.saved_x;
        let y = b
            .saved_y
            .saturating_sub(i64::try_from(b.ybase).map_err(|_| TerminalError::InvalidDimensions)?)
            .max(0);
        self.attributes.fg = b.saved_cur_attr_data.fg;
        self.attributes.bg = b.saved_cur_attr_data.bg;
        for (level, charset) in b.saved_charsets.iter().enumerate() {
            self.charset
                .set_g_charset(level, charset.clone())
                .map_err(|e| TerminalError::Buffer(e.to_string()))?;
        }
        self.charset.set_g_level(b.saved_glevel);
        {
            let mut modes = self.core.dec_private_modes.borrow_mut();
            modes.origin = b.saved_origin_mode;
            modes.wraparound = b.saved_wraparound_mode;
        }
        drop(b);
        let mut b = active.borrow_mut();
        b.x = x.min(self.cols() - 1);
        b.y = if self.core.dec_private_modes.borrow().origin {
            y.clamp(b.scroll_top as i64, b.scroll_bottom as i64) as usize
        } else {
            y.min((self.rows() - 1) as i64) as usize
        };
        self.dirty_row(b.y);
        Ok(())
    }
    pub fn reset_state(&mut self) -> Result<(), TerminalError> {
        self.buffer_service
            .reset()
            .map_err(|e| TerminalError::Buffer(e.to_string()))?;
        self.core.reset();
        self.mouse.reset()?;
        self.charset.reset();
        self.attributes = AttributeData::default();
        self.dirty_all();
        Ok(())
    }
}
impl From<crate::mouse_state::MouseStateError> for TerminalError {
    fn from(e: crate::mouse_state::MouseStateError) -> Self {
        Self::Mouse(e)
    }
}
impl From<crate::log_service::LogError> for TerminalError {
    fn from(e: crate::log_service::LogError) -> Self {
        Self::Log(e)
    }
}
