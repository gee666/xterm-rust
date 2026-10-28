// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Cell/column changes are local. Row changes release state before observers.
use super::{csi::count, TerminalError, TerminalState};
use crate::{
    buffer::{AttributeData, LineRef},
    parser::params::Params,
};
use std::{cell::RefCell, rc::Rc};

impl TerminalState {
    pub(super) fn dispatch_edits(
        &mut self,
        command: &str,
        params: &Params,
        selective: bool,
    ) -> Result<bool, TerminalError> {
        match command {
            "@" | "P" => self.edit_characters(params, command == "@")?,
            " @" | " A" | "'}" | "'~" => self.edit_columns(params, command)?,
            _ => return self.dispatch_erase(command, params, selective),
        }
        Ok(true)
    }
    fn edit_characters(&mut self, params: &Params, insert: bool) -> Result<(), TerminalError> {
        self.restrict_csi_cursor(self.cols() - 1)?;
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        if let Some(line) = b.lines.get(b.ybase + b.y) {
            let fill = b.get_null_cell(Some(&self.erase_attributes()));
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
    fn edit_columns(&mut self, params: &Params, command: &str) -> Result<(), TerminalError> {
        let active = self.active_buffer();
        let b = active.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        if b.y < b.scroll_top || b.y > b.scroll_bottom {
            return Ok(());
        }
        let position = if command.starts_with(' ') { 0 } else { b.x };
        let insert = matches!(command, " A" | "'}");
        let amount = count(params)?;
        let fill = b.get_null_cell(Some(&self.erase_attributes()));
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

pub(super) fn rows(
    shared: &Rc<RefCell<TerminalState>>,
    params: &Params,
    command: &str,
) -> Result<(), TerminalError> {
    match command {
        "L" | "M" => edit_lines(shared, params, command == "L"),
        "S" | "T" | "^" => scroll_rows(shared, params, command != "S"),
        _ => Err(TerminalError::InvalidParameter(
            "unknown row editing command",
        )),
    }
}
fn active(
    shared: &Rc<RefCell<TerminalState>>,
) -> Result<Rc<RefCell<crate::buffer::Buffer>>, TerminalError> {
    Ok(shared
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .active_buffer())
}
fn splice(
    shared: &Rc<RefCell<TerminalState>>,
    index: usize,
    count: usize,
    rows: &[LineRef],
) -> Result<(), TerminalError> {
    let buffer = active(shared)?;
    let lines = buffer
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .lines
        .share();
    lines
        .splice(index, count, rows)
        .map_err(crate::buffer::BufferError::from)?;
    Ok(())
}
fn blank(shared: &Rc<RefCell<TerminalState>>, default: bool) -> Result<LineRef, TerminalError> {
    let state = shared.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    let buffer = state.active_buffer();
    let attrs = if default {
        AttributeData::default()
    } else {
        state.erase_attributes()
    };
    let result = buffer
        .try_borrow()
        .map_err(|_| TerminalError::Borrowed)?
        .get_blank_line(&attrs, false);
    Ok(result)
}
fn edit_lines(
    shared: &Rc<RefCell<TerminalState>>,
    params: &Params,
    insert: bool,
) -> Result<(), TerminalError> {
    {
        let mut state = shared
            .try_borrow_mut()
            .map_err(|_| TerminalError::Borrowed)?;
        let max = state.cols() - 1;
        state.restrict_csi_cursor(max)?;
    }
    let buffer = active(shared)?;
    let b = buffer.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    if b.y < b.scroll_top || b.y > b.scroll_bottom {
        return Ok(());
    }
    let start = b.ybase + b.y;
    let end = b.ybase + b.scroll_bottom;
    drop(b);
    // The source captures absolute indices, but reloads the receiver's buffer
    // after each callback. A reset can therefore create sparse insertion slots.
    for _ in 0..count(params)? {
        let (remove, add) = if insert { (end, start) } else { (start, end) };
        splice(shared, remove, 1, &[])?;
        let line = blank(shared, false)?;
        splice(shared, add, 0, &[line])?;
    }
    let buffer = active(shared)?;
    let mut b = buffer
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    let mut state = shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    state.dirty_row(b.y);
    state.dirty_row(b.scroll_bottom);
    b.x = 0;
    Ok(())
}
fn scroll_rows(
    shared: &Rc<RefCell<TerminalState>>,
    params: &Params,
    down: bool,
) -> Result<(), TerminalError> {
    for _ in 0..count(params)? {
        let buffer = active(shared)?;
        let b = buffer.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let remove = b.ybase + if down { b.scroll_bottom } else { b.scroll_top };
        drop(b);
        splice(shared, remove, 1, &[])?;
        let buffer = active(shared)?;
        let b = buffer.try_borrow().map_err(|_| TerminalError::Borrowed)?;
        let add = b.ybase + if down { b.scroll_top } else { b.scroll_bottom };
        drop(b);
        // SD deliberately uses defaults; SU retains the current BCE color.
        let line = blank(shared, down)?;
        splice(shared, add, 0, &[line])?;
    }
    let buffer = active(shared)?;
    let b = buffer.try_borrow().map_err(|_| TerminalError::Borrowed)?;
    let mut state = shared
        .try_borrow_mut()
        .map_err(|_| TerminalError::Borrowed)?;
    state.dirty_row(b.scroll_top);
    state.dirty_row(b.scroll_bottom);
    Ok(())
}
