// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Cell mutations for one print run. User callbacks only run outside borrows.
use super::{PrintRun, RcLine, TerminalError};
use crate::unicode::UnicodeCharProperties;

impl PrintRun {
    /// False is the source's defensive return when resize removed the next row.
    pub(super) fn render(
        &mut self,
        code: u32,
        current: UnicodeCharProperties,
        old_width: usize,
    ) -> Result<bool, TerminalError> {
        let width = usize::from(current.width());
        let active = self.active();
        let x = active.try_borrow().map_err(|_| TerminalError::Borrowed)?.x;
        let edge = self
            .cols
            .checked_add(old_width)
            .ok_or(TerminalError::InvalidDimensions)?;
        if x.checked_add(width)
            .ok_or(TerminalError::InvalidDimensions)?
            > edge
        {
            if self.wraparound {
                if !self.wrap(old_width)? {
                    return Ok(false);
                }
            } else {
                active
                    .try_borrow_mut()
                    .map_err(|_| TerminalError::Borrowed)?
                    .x = self.cols - 1;
                if width == 2 {
                    return Ok(true);
                }
            }
        }
        // A callback may have changed the active buffer while scrolling.
        let active = self.active();
        let x = active.try_borrow().map_err(|_| TerminalError::Borrowed)?.x;
        let attrs = self.attrs()?;
        if current.should_join() && x != 0 {
            let offset = if self
                .row
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .get_width(x - 1)
                .is_some_and(|width| width != 0)
            {
                1
            } else {
                2
            };
            self.row
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .add_codepoint_to_cell(x.saturating_sub(offset), code, width as i32);
            for column in 0..width.saturating_sub(old_width) {
                self.row
                    .try_borrow_mut()
                    .map_err(|_| TerminalError::Borrowed)?
                    .set_cell_from_codepoint(x + column, 0, 0, &attrs);
                self.advance_cursor()?;
            }
            return Ok(true);
        }
        if self.insert {
            let fill = active
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .get_null_cell_snapshot(Some(&attrs));
            let mut row = self
                .row
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            row.insert_cells(x, width.saturating_sub(old_width), &fill);
            if row.get_width(self.cols - 1) == Some(2) {
                row.set_cell_from_codepoint(self.cols - 1, 0, 1, &attrs);
            }
        }
        self.row
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?
            .set_cell_from_codepoint(x, code, width as i32, &attrs);
        self.advance_cursor()?;
        for column in 1..width {
            self.row
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .set_cell_from_codepoint(x + column, 0, 0, &attrs);
            self.advance_cursor()?;
        }
        Ok(true)
    }

    fn advance_cursor(&self) -> Result<(), TerminalError> {
        let active = self.active();
        let mut buffer = active
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        buffer.x = buffer
            .x
            .checked_add(1)
            .ok_or(TerminalError::InvalidDimensions)?;
        Ok(())
    }

    fn wrap(&mut self, old_width: usize) -> Result<bool, TerminalError> {
        let old_row = RcLine::clone(&self.row);
        let active = self.active();
        let (old_col, scroll) = {
            let mut buffer = active
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?;
            let old_col = buffer.x.saturating_sub(old_width);
            buffer.x = old_width;
            buffer.y = buffer
                .y
                .checked_add(1)
                .ok_or(TerminalError::InvalidDimensions)?;
            let bottom = buffer
                .scroll_bottom
                .checked_add(1)
                .ok_or(TerminalError::InvalidDimensions)?;
            let scroll = buffer.y == bottom;
            if scroll {
                buffer.y -= 1;
            } else {
                buffer.y = buffer.y.min(self.service.rows() - 1);
                let index = buffer
                    .ybase
                    .checked_add(buffer.y)
                    .ok_or(TerminalError::InvalidDimensions)?;
                buffer.set_wrapped(index, true)?;
            }
            (old_col, scroll)
        };
        if scroll {
            // This helper releases terminal/Buffer borrows before marker/scroll
            // callbacks and preserves their committed partial state.
            super::super::controls::scroll_state(&self.shared, true)?;
        }
        let Some(row) = self.row_for_active()? else {
            return Ok(false);
        };
        self.row = row;
        if old_width > 0 {
            // Snapshot also handles source and destination being the same row,
            // without holding immutable and mutable RefCell borrows together.
            let saved = old_row
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .clone();
            self.row
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .copy_cells_from(&saved, old_col, 0, old_width, false);
        }
        let attrs = self.attrs()?;
        for column in old_col..self.cols {
            old_row
                .try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .set_cell_from_codepoint(column, 0, 1, &attrs);
        }
        Ok(true)
    }
}
