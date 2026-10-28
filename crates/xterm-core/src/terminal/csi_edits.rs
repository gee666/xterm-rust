// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Cell, row and column edits preserve the source's per-operation list events.
use super::{csi::count, TerminalError, TerminalState};
use crate::{buffer::AttributeData, parser::params::Params};

impl TerminalState {
    pub(super) fn dispatch_edits(
        &mut self,
        final_byte: &str,
        params: &Params,
        selective: bool,
    ) -> Result<bool, TerminalError> {
        match final_byte {
            "@" | "P" => self.edit_characters(params, final_byte == "@")?,
            "L" | "M" => self.edit_lines(params, final_byte == "L")?,
            "S" | "T" | "^" => self.scroll_csi_rows(params, final_byte != "S")?,
            " @" | " A" | "'}" | "'~" => self.edit_columns(params, final_byte)?,
            _ => return self.dispatch_erase(final_byte, params, selective),
        }
        Ok(true)
    }
    fn edit_characters(&mut self, params: &Params, insert: bool) -> Result<(), TerminalError> {
        self.restrict_csi_cursor(self.cols() - 1)?;
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        if let Some(line) = b.lines.get(b.ybase + b.y) {
            let fill = b.get_null_cell_snapshot(Some(&self.erase_attributes()));
            let mut line = line.try_borrow_mut().map_err(|_| TerminalError::Borrowed)?;
            if insert {
                line.insert_cells(b.x, count(params)?, &fill);
            } else {
                line.delete_cells(b.x, count(params)?, &fill);
            }
            self.dirty_row(b.y);
        }
        Ok(())
    }
    fn edit_lines(&mut self, params: &Params, insert: bool) -> Result<(), TerminalError> {
        self.restrict_csi_cursor(self.cols() - 1)?;
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        if b.y < b.scroll_top || b.y > b.scroll_bottom {
            return Ok(());
        }
        let start = b.ybase + b.y;
        let end = b.ybase + b.scroll_bottom;
        drop(b);
        for _ in 0..count(params)? {
            let (remove, add) = if insert { (end, start) } else { (start, end) };
            self.splice_csi_rows(remove, 1, &[])?;
            let active = self.active_buffer();
            let blank = active
                .try_borrow()
                .map_err(|_| TerminalError::Borrowed)?
                .get_blank_line(&self.erase_attributes(), false);
            self.splice_csi_rows(add, 0, &[blank])?;
        }
        let current = self.active_buffer();
        let b = current.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        self.dirty_row(b.y);
        self.dirty_row(b.scroll_bottom);
        drop(b);
        current
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?
            .x = 0;
        Ok(())
    }
    fn scroll_csi_rows(&mut self, params: &Params, down: bool) -> Result<(), TerminalError> {
        for _ in 0..count(params)? {
            let active = self.active_buffer();
            let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
            let remove = b.ybase + if down { b.scroll_bottom } else { b.scroll_top };
            drop(b);
            self.splice_csi_rows(remove, 1, &[])?;
            let active = self.active_buffer();
            let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
            let add = b.ybase + if down { b.scroll_top } else { b.scroll_bottom };
            // SD intentionally uses default attributes while SU uses BCE attributes.
            let attrs = if down {
                AttributeData::default()
            } else {
                self.erase_attributes()
            };
            let blank = b.get_blank_line(&attrs, false);
            drop(b);
            self.splice_csi_rows(add, 0, &[blank])?;
        }
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        self.dirty_row(b.scroll_top);
        self.dirty_row(b.scroll_bottom);
        Ok(())
    }
    fn splice_csi_rows(
        &self,
        index: usize,
        count: usize,
        rows: &[crate::buffer::LineRef],
    ) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let lines = active
            .try_borrow()
            .map_err(|_| TerminalError::Borrowed)?
            .lines
            .share();
        // Buffer observers can mutate the buffer during list callbacks.
        lines
            .splice(index, count, rows)
            .map_err(crate::buffer::BufferError::from)?;
        Ok(())
    }
    fn edit_columns(&mut self, params: &Params, command: &str) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        if b.y < b.scroll_top || b.y > b.scroll_bottom {
            return Ok(());
        }
        let position = if command.starts_with(' ') { 0 } else { b.x };
        let insert = matches!(command, " A" | "'}");
        let amount = count(params)?;
        let fill = b.get_null_cell_snapshot(Some(&self.erase_attributes()));
        for y in b.scroll_top..=b.scroll_bottom {
            let line = b.lines.get(b.ybase + y).ok_or(TerminalError::MissingLine)?;
            let mut line = line.try_borrow_mut().map_err(|_| TerminalError::Borrowed)?;
            line.is_wrapped = false;
            if insert {
                line.insert_cells(position, amount, &fill);
            } else {
                line.delete_cells(position, amount, &fill);
            }
        }
        self.dirty_row(b.scroll_top);
        self.dirty_row(b.scroll_bottom);
        Ok(())
    }
}
