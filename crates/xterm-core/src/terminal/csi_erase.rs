// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! ED, EL, ECH and DECSCA, including protection, BCE and scrollback effects.
use super::{
    csi::{count, parameter},
    TerminalError, TerminalEvent, TerminalState,
};
use crate::{buffer::constants::bg_flags::PROTECTED, parser::params::Params};
use std::rc::Rc;

impl TerminalState {
    pub(super) fn dispatch_erase(
        &mut self,
        command: &str,
        params: &Params,
        selective: bool,
    ) -> Result<bool, TerminalError> {
        match command {
            "J" => self.erase_display(parameter(params, 0, 0), selective)?,
            "K" => self.erase_line(parameter(params, 0, 0), selective)?,
            "X" => self.erase_characters(params)?,
            "\"q" => match parameter(params, 0, 0) {
                1 => self.attributes.bg |= PROTECTED,
                0 | 2 => self.attributes.bg &= !PROTECTED,
                _ => {}
            },
            _ => return Ok(false),
        }
        Ok(true)
    }

    fn erase_row_range(
        &mut self,
        y: usize,
        start: usize,
        end: usize,
        clear_wrap: bool,
        selective: bool,
    ) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let Some(line) = b.lines.get(b.ybase + y) else {
            return Ok(());
        };
        let fill = b.get_null_cell(Some(&self.erase_attributes()));
        drop(b);
        let mut line = line.try_borrow_mut().map_err(|_| TerminalError::Borrowed)?;
        line.replace_cells(start, end, &fill, selective);
        if clear_wrap {
            line.is_wrapped = false;
        }
        Ok(())
    }

    fn reset_erase_row(&mut self, y: usize, selective: bool) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let row = b.ybase + y;
        let Some(line) = b.lines.get(row) else {
            return Ok(());
        };
        let fill = b.get_null_cell(Some(&self.erase_attributes()));
        drop(b);
        line.try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?
            .fill(&fill, selective);
        // Marker callbacks run after fill, but before the wrapped flag changes.
        crate::buffer::Buffer::clear_markers_shared(
            &active,
            i64::try_from(row).map_err(|_| TerminalError::InvalidDimensions)?,
        )?;
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        if let Some(line) = b.lines.get(b.ybase + y) {
            line.try_borrow_mut()
                .map_err(|_| TerminalError::Borrowed)?
                .is_wrapped = false;
        }
        Ok(())
    }

    fn erase_line(&mut self, mode: i64, selective: bool) -> Result<(), TerminalError> {
        self.restrict_csi_cursor(self.cols())?;
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let (x, y) = (b.x, b.y);
        drop(b);
        match mode {
            0 => self.erase_row_range(y, x, self.cols(), x == 0, selective)?,
            1 => self.erase_row_range(y, 0, x + 1, false, selective)?,
            2 => self.erase_row_range(y, 0, self.cols(), true, selective)?,
            _ => {}
        }
        self.dirty_row(y);
        Ok(())
    }

    fn erase_display(&mut self, mode: i64, selective: bool) -> Result<(), TerminalError> {
        self.restrict_csi_cursor(self.cols())?;
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let (x, y) = (b.x, b.y);
        drop(b);
        match mode {
            0 => {
                self.dirty_row(y);
                self.erase_row_range(y, x, self.cols(), x == 0, selective)?;
                for row in y + 1..self.rows() {
                    self.reset_erase_row(row, selective)?;
                }
                self.dirty_row(self.rows());
            }
            1 => {
                self.dirty_row(y);
                self.erase_row_range(y, 0, x + 1, true, selective)?;
                if x + 1 >= self.cols() {
                    // Source passes an absolute index here, without ybase.
                    if let Some(next) = active
                        .try_borrow()
                        .map_err(|_| TerminalError::Borrowed)?
                        .lines
                        .get(y + 1)
                    {
                        next.try_borrow_mut()
                            .map_err(|_| TerminalError::Borrowed)?
                            .is_wrapped = false;
                    }
                }
                for row in (0..y).rev() {
                    self.reset_erase_row(row, selective)?;
                }
                self.dirty_row(0);
            }
            2 if self.option_bool("scrollOnEraseInDisplay")? => {
                self.dirty_all();
                let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
                let mut scroll_count = 0;
                for row in (0..self.rows()).rev() {
                    if let Some(line) = b.lines.get(b.ybase + row) {
                        if line
                            .try_borrow()
                            .map_err(|_| TerminalError::Borrowed)?
                            .get_trimmed_length()
                            != 0
                        {
                            scroll_count = row + 1;
                            break;
                        }
                    }
                }
                drop(b);
                for _ in 0..scroll_count {
                    self.buffer_service
                        .scroll(&self.erase_attributes(), false)?;
                }
            }
            2 => {
                self.dirty_row(self.rows() - 1);
                for row in (0..self.rows()).rev() {
                    self.reset_erase_row(row, selective)?;
                }
                self.dirty_row(0);
            }
            3 => {
                let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
                let scrollback = b.lines.len().saturating_sub(self.rows());
                let lines = b.lines.share();
                drop(b);
                if scrollback > 0 {
                    // Trim observers must see old bases and be able to inspect
                    // the buffer without a live RefMut held by this command.
                    lines
                        .trim_start(scrollback)
                        .map_err(crate::buffer::BufferError::from)?;
                    let mut b = active
                        .try_borrow_mut()
                        .map_err(|_| TerminalError::Borrowed)?;
                    b.ybase = b.ybase.saturating_sub(scrollback);
                    b.ydisp = b.ydisp.saturating_sub(scrollback);
                    drop(b);
                    if Rc::ptr_eq(&active, &self.buffers.normal()) {
                        self.buffer_service.is_user_scrolling.set(false);
                    }
                    self.events.borrow_mut().push(TerminalEvent::Scroll(0));
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn erase_characters(&mut self, params: &Params) -> Result<(), TerminalError> {
        self.restrict_csi_cursor(self.cols() - 1)?;
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let (x, y) = (b.x, b.y);
        let end = x
            .checked_add(count(params)?)
            .ok_or(TerminalError::InvalidDimensions)?;
        let exists = b.lines.get(b.ybase + y).is_some();
        drop(b);
        if exists {
            self.erase_row_range(y, x, end, false, false)?;
            self.dirty_row(y);
        }
        Ok(())
    }
}
