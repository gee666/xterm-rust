use super::{HeadlessError, Terminal};
use crate::terminal::TerminalError;
use std::rc::Rc;

impl Terminal {
    /// No engine/state borrow crosses scroll listeners. The source delegates
    /// directly to BufferService and does not mark a parser dirty range here.
    pub fn scroll_lines(&self, amount: i64) -> Result<(), HeadlessError> {
        let service = self.with_state(|state| Rc::clone(&state.buffer_service))?;
        service
            .scroll_lines(amount, false)
            .map_err(TerminalError::from)?;
        self.flush_notifications()
    }
    pub fn scroll_pages(&self, count: i64) -> Result<(), HeadlessError> {
        let height =
            i64::try_from(self.rows()? - 1).map_err(|_| TerminalError::InvalidDimensions)?;
        self.scroll_lines(count.saturating_mul(height))
    }
    pub fn scroll_to_top(&self) -> Result<(), HeadlessError> {
        self.scroll_lines(-self.viewport_position()?)
    }
    pub fn scroll_to_bottom(&self) -> Result<(), HeadlessError> {
        let buffer = self.with_state(|state| state.active_buffer())?;
        let delta = {
            let buffer = buffer.try_borrow().map_err(|_| HeadlessError::Borrowed)?;
            i64::try_from(buffer.ybase).map_err(|_| TerminalError::InvalidDimensions)?
                - i64::try_from(buffer.ydisp).map_err(|_| TerminalError::InvalidDimensions)?
        };
        self.scroll_lines(delta)
    }
    pub fn scroll_to_line(&self, line: usize) -> Result<(), HeadlessError> {
        match i64::try_from(line) {
            Ok(line) => self.scroll_to_line_signed(line),
            // The engine bounds scrollback below i64::MAX. Larger positive
            // native indices therefore clamp to the source's bottom boundary.
            Err(_) => self.scroll_to_bottom(),
        }
    }
    /// Signed counterpart of JavaScript scrollToLine. Saturation only affects
    /// targets already outside the viewport, which clamp to the same boundary.
    pub fn scroll_to_line_signed(&self, line: i64) -> Result<(), HeadlessError> {
        let delta = line.saturating_sub(self.viewport_position()?);
        if delta == 0 {
            return Ok(());
        }
        self.scroll_lines(delta)
    }
    fn viewport_position(&self) -> Result<i64, HeadlessError> {
        let buffer = self.with_state(|state| state.active_buffer())?;
        let position = buffer
            .try_borrow()
            .map_err(|_| HeadlessError::Borrowed)?
            .ydisp;
        Ok(i64::try_from(position).map_err(|_| TerminalError::InvalidDimensions)?)
    }
}
