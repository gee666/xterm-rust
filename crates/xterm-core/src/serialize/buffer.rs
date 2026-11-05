// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{BufferAccess, SerializeError};
use crate::buffer::{Buffer, CellData, LineRef};

pub(super) fn line_length(line: Option<LineRef>) -> Result<Option<usize>, SerializeError> {
    let Some(line) = line else {
        return Ok(None);
    };
    let line = line
        .try_borrow()
        .map_err(|_| SerializeError::BorrowedLine)?;
    Ok(Some(line.len()))
}
pub(super) fn line_cell(
    line: Option<LineRef>,
    col: usize,
) -> Result<Option<CellData>, SerializeError> {
    let Some(line) = line else {
        return Ok(None);
    };
    let line = line
        .try_borrow()
        .map_err(|_| SerializeError::BorrowedLine)?;
    Ok(line.get_cell(col))
}
impl BufferAccess for Buffer {
    fn len(&self) -> usize {
        self.lines.len()
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        line_length(self.lines.get(row))
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        line_cell(self.lines.get(row), col)
    }
    fn capture_numeric_line(&self, row: f64) -> Result<Option<usize>, SerializeError> {
        line_length(self.lines.get_number(row))
    }
    fn captured_numeric_cell(
        &self,
        row: f64,
        col: usize,
    ) -> Result<Option<CellData>, SerializeError> {
        line_cell(self.lines.get_number(row), col)
    }
    fn is_wrapped(&self, row: usize) -> Result<bool, SerializeError> {
        let Some(line) = self.lines.get(row) else {
            return Ok(false);
        };
        let line = line
            .try_borrow()
            .map_err(|_| SerializeError::BorrowedLine)?;
        Ok(line.is_wrapped)
    }
    fn cursor_x(&self) -> usize {
        self.x
    }
    fn cursor_y(&self) -> usize {
        self.y
    }
    fn base_y(&self) -> usize {
        self.ybase
    }
}
