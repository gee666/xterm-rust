// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{Buffer, BufferError};
use crate::buffer::{AttributeData, BufferLine};
use std::{cell::RefCell, rc::Rc};

impl Buffer {
    /// Resize internal state; the owning service updates host_dimensions first.
    pub fn resize(&mut self, new_cols: usize, new_rows: usize) -> Result<(), BufferError> {
        if new_cols == 0 || new_rows == 0 {
            return Err(BufferError::InvalidDimensions);
        }
        let null = self.get_null_cell(Some(&AttributeData::default()));
        let mut dirty = 0;
        let new_max = self.correct_buffer_length(new_rows)?;
        if new_max > self.lines.max_length() {
            self.lines.set_max_length(new_max)?;
        }
        if !self.lines.is_empty() {
            if self.cols < new_cols {
                for i in 0..self.lines.len() {
                    dirty += usize::from(
                        self.line(i)?
                            .try_borrow_mut()
                            .map_err(|_| BufferError::BorrowedLine)?
                            .resize(new_cols, &null),
                    );
                }
            }
            let mut add_to_y = 0;
            if self.rows < new_rows {
                let pty = self.windows_pty()?;
                for _ in self.rows..new_rows {
                    if self.lines.len()
                        < new_rows
                            .checked_add(self.ybase)
                            .ok_or(BufferError::SizeOverflow)?
                    {
                        if pty.backend.is_some() || pty.build_number.is_some() {
                            self.lines.push(Rc::new(RefCell::new(BufferLine::new(
                                new_cols,
                                Some(&null),
                                false,
                            ))))?;
                        } else if self.ybase > 0
                            && self.lines.len()
                                <= self
                                    .ybase
                                    .checked_add(self.y)
                                    .and_then(|n| n.checked_add(add_to_y))
                                    .and_then(|n| n.checked_add(1))
                                    .ok_or(BufferError::SizeOverflow)?
                        {
                            self.ybase -= 1;
                            add_to_y += 1;
                            self.ydisp = self.ydisp.saturating_sub(1);
                        } else {
                            self.lines.push(Rc::new(RefCell::new(BufferLine::new(
                                new_cols,
                                Some(&null),
                                false,
                            ))))?;
                        }
                    }
                }
            } else {
                for _ in (new_rows..self.rows).rev() {
                    if self.lines.len()
                        > new_rows
                            .checked_add(self.ybase)
                            .ok_or(BufferError::SizeOverflow)?
                    {
                        if self.lines.len()
                            > self
                                .ybase
                                .checked_add(self.y)
                                .and_then(|n| n.checked_add(1))
                                .ok_or(BufferError::SizeOverflow)?
                        {
                            self.lines.pop()?;
                        } else {
                            self.ybase =
                                self.ybase.checked_add(1).ok_or(BufferError::SizeOverflow)?;
                            self.ydisp =
                                self.ydisp.checked_add(1).ok_or(BufferError::SizeOverflow)?;
                        }
                    }
                }
            }
            if new_max < self.lines.max_length() {
                let trim = self.lines.len().saturating_sub(new_max);
                if trim > 0 {
                    self.lines.trim_start(trim)?;
                    self.ybase = self.ybase.saturating_sub(trim);
                    self.ydisp = self.ydisp.saturating_sub(trim);
                    self.saved_y = self
                        .saved_y
                        .saturating_sub(i64::try_from(trim).map_err(|_| BufferError::SizeOverflow)?)
                        .max(0);
                }
                self.lines.set_max_length(new_max)?;
            }
            self.x = self.x.min(new_cols - 1);
            self.y = self
                .y
                .min(new_rows - 1)
                .checked_add(add_to_y)
                .ok_or(BufferError::SizeOverflow)?;
            self.saved_x = self.saved_x.min(new_cols - 1);
            self.scroll_top = 0;
        }
        self.scroll_bottom = new_rows - 1;
        if self.is_reflow_enabled()? {
            self.reflow(new_cols, new_rows)?;
            if self.cols > new_cols {
                for i in 0..self.lines.len() {
                    dirty += usize::from(
                        self.line(i)?
                            .try_borrow_mut()
                            .map_err(|_| BufferError::BorrowedLine)?
                            .resize(new_cols, &null),
                    );
                }
            }
        }
        self.cols = new_cols;
        self.rows = new_rows;
        if !self.lines.is_empty() {
            self.y = self.y.min(
                self.lines
                    .len()
                    .saturating_sub(self.ybase)
                    .saturating_sub(1),
            );
        }
        self.cleanup_pending = false;
        self.cleanup_events.fire(&false)?;
        if dirty as f64 > 0.1 * self.lines.len() as f64 {
            self.cleanup_position = 0;
            self.cleanup_pending = true;
            self.cleanup_events.fire(&true)?;
        }
        Ok(())
    }
}
