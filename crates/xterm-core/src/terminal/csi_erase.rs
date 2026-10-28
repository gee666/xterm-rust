// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! ED callbacks use shared state checkpoints; local erasure needs no callouts.
use super::{
    csi::{count, parameter},
    TerminalError, TerminalEvent, TerminalState,
};
use crate::{
    buffer::{constants::bg_flags::PROTECTED, Buffer},
    parser::params::Params,
};
use std::{cell::RefCell, rc::Rc};
type Shared = Rc<RefCell<TerminalState>>;

impl TerminalState {
    pub(super) fn dispatch_erase(
        &mut self,
        command: &str,
        params: &Params,
        selective: bool,
    ) -> Result<bool, TerminalError> {
        match command {
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
        &self,
        y: usize,
        start: usize,
        end: usize,
        clear: bool,
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
        if clear {
            line.is_wrapped = false;
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
fn active(shared: &Shared) -> Result<Rc<RefCell<Buffer>>, TerminalError> {
    Ok(shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .active_buffer())
}
fn dirty(shared: &Shared, row: usize) -> Result<(), TerminalError> {
    shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?
        .dirty_row(row);
    Ok(())
}
fn rows(shared: &Shared) -> Result<usize, TerminalError> {
    Ok(shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .rows())
}
fn erase_range(
    shared: &Shared,
    y: usize,
    start: usize,
    end: usize,
    clear: bool,
    selective: bool,
) -> Result<(), TerminalError> {
    shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .erase_row_range(y, start, end, clear, selective)
}
fn reset_row(shared: &Shared, y: usize, selective: bool) -> Result<(), TerminalError> {
    let (buffer, row, line, fill) = {
        let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let buffer = state.active_buffer();
        let b = buffer.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let row = b.ybase + y;
        let Some(line) = b.lines.get(row) else {
            return Ok(());
        };
        let fill = b.get_null_cell(Some(&state.erase_attributes()));
        drop(b);
        (buffer, row, line, fill)
    };
    line.try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?
        .fill(&fill, selective);
    Buffer::clear_markers_shared(
        &buffer,
        i64::try_from(row).map_err(|_| TerminalError::InvalidDimensions)?,
    )?;
    // Reset/activation can replace the receiver. Source clears wrapping on the
    // current buffer after marker callbacks, not on the previously filled row.
    let buffer = active(shared)?;
    let b = buffer.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    b.set_wrapped(b.ybase + y, false)?;
    Ok(())
}

pub(super) fn erase_display(
    shared: &Shared,
    params: &Params,
    selective: bool,
) -> Result<(), TerminalError> {
    {
        let mut state = shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        let max = state.cols();
        state.restrict_csi_cursor(max)?;
    }
    let buffer = active(shared)?;
    let b = buffer.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    let (x, y) = (b.x, b.y);
    drop(b);
    match parameter(params, 0, 0) {
        0 => erase_below(shared, x, y, selective)?,
        1 => erase_above(shared, x, y, selective)?,
        2 => erase_all(shared, selective)?,
        3 => erase_scrollback(shared)?,
        _ => {}
    }
    Ok(())
}
fn erase_below(shared: &Shared, x: usize, y: usize, selective: bool) -> Result<(), TerminalError> {
    dirty(shared, y)?;
    let cols = shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .cols();
    erase_range(shared, y, x, cols, x == 0, selective)?;
    let mut row = y + 1;
    // Source tests the live row count after each marker callback.
    while row < rows(shared)? {
        reset_row(shared, row, selective)?;
        row += 1;
    }
    dirty(shared, row)
}
fn erase_above(shared: &Shared, x: usize, y: usize, selective: bool) -> Result<(), TerminalError> {
    dirty(shared, y)?;
    erase_range(shared, y, 0, x + 1, true, selective)?;
    let cols = shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .cols();
    if x + 1 >= cols {
        // The reference passes an absolute index here, deliberately no ybase.
        active(shared)?
            .try_borrow()
            .map_err(|_| TerminalError::Borrowed)?
            .set_wrapped(y + 1, false)?;
    }
    for row in (0..y).rev() {
        reset_row(shared, row, selective)?;
    }
    dirty(shared, 0)
}
fn erase_all(shared: &Shared, selective: bool) -> Result<(), TerminalError> {
    let scroll = shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .option_bool("scrollOnEraseInDisplay")?;
    let row_count = rows(shared)?;
    if !scroll {
        dirty(shared, row_count - 1)?;
        for row in (0..row_count).rev() {
            reset_row(shared, row, selective)?;
        }
        return dirty(shared, 0);
    }
    shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?
        .dirty_all();
    let buffer = active(shared)?;
    let b = buffer.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    let mut scroll_count = 0;
    for row in (0..row_count).rev() {
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
        let (service, attrs) = {
            let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
            (Rc::clone(&state.buffer_service), state.erase_attributes())
        };
        service.scroll(&attrs, false)?;
        super::dispatch_events(shared)?;
    }
    Ok(())
}
fn erase_scrollback(shared: &Shared) -> Result<(), TerminalError> {
    let buffer = active(shared)?;
    let b = buffer.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    let trim = b.lines.len().saturating_sub(rows(shared)?);
    let lines = b.lines.share();
    drop(b);
    if trim == 0 {
        return Ok(());
    }
    lines
        .trim_start(trim)
        .map_err(crate::buffer::BufferError::from)?;
    let buffer = active(shared)?;
    let mut b = buffer
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    b.ybase = b.ybase.saturating_sub(trim);
    b.ydisp = b.ydisp.saturating_sub(trim);
    drop(b);
    let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    if Rc::ptr_eq(&buffer, &state.buffers.normal()) {
        state.buffer_service.is_user_scrolling.set(false);
    }
    state
        .events
        .borrow_mut()
        .push(TerminalEvent::InputScroll(0));
    Ok(())
}
