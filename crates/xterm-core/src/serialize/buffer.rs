// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{BufferAccess, SerializeError};
use crate::buffer::{Buffer, CellData};

impl BufferAccess for Buffer {
    fn len(&self) -> usize {
        self.lines.len()
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        let Some(line) = self.lines.get(row) else {
            return Ok(None);
        };
        let line = line
            .try_borrow()
            .map_err(|_| SerializeError::BorrowedLine)?;
        Ok(Some(line.len()))
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        let Some(line) = self.lines.get(row) else {
            return Ok(None);
        };
        let line = line
            .try_borrow()
            .map_err(|_| SerializeError::BorrowedLine)?;
        Ok(line.get_cell(col))
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
