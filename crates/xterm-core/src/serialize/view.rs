// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
use super::{BufferAccess, SerializeError};
use crate::{
    buffer::{CellData, LineRef},
    public_api::buffer::BufferApiView,
};
use std::rc::Rc;

/// Capture metrics after header callbacks, but read cells through the actual
/// public view. Reset may reinitialize that same view to a new Buffer, while an
/// alternate-screen activation leaves the captured normal view unchanged.
pub struct View {
    view: Rc<BufferApiView>,
    length: usize,
    x: usize,
    y: usize,
    base: usize,
}
impl View {
    pub fn new(view: Rc<BufferApiView>) -> Result<Self, SerializeError> {
        Ok(Self {
            length: view.len()?,
            x: view.cursor_x()?,
            y: view.cursor_y()?,
            base: view.base_y()?,
            view,
        })
    }
    fn line(&self, row: usize) -> Result<Option<LineRef>, SerializeError> {
        self.view.get_line(row).map_err(Into::into)
    }
}
impl BufferAccess for View {
    fn len(&self) -> usize {
        self.length
    }
    fn cursor_x(&self) -> usize {
        self.x
    }
    fn cursor_y(&self) -> usize {
        self.y
    }
    fn base_y(&self) -> usize {
        self.base
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        let Some(line) = self.line(row)? else {
            return Ok(None);
        };
        let line = line
            .try_borrow()
            .map_err(|_| SerializeError::BorrowedLine)?;
        Ok(Some(line.len()))
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        let Some(line) = self.line(row)? else {
            return Ok(None);
        };
        let line = line
            .try_borrow()
            .map_err(|_| SerializeError::BorrowedLine)?;
        Ok(line.get_cell(col))
    }
    fn is_wrapped(&self, row: usize) -> Result<bool, SerializeError> {
        let Some(line) = self.line(row)? else {
            return Ok(false);
        };
        let line = line
            .try_borrow()
            .map_err(|_| SerializeError::BorrowedLine)?;
        Ok(line.is_wrapped)
    }
}
