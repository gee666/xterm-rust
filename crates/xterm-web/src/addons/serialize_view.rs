//! Number-indexed reads through the actual shared Rust public buffer view.
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::JsValue;
use xterm_core::{
    buffer::{CellData, LineRef},
    public_api::{
        buffer::BufferApiView,
        line_numbers::{get_cell_number, NumericCell},
    },
    serialize::{BufferAccess, SerializeError},
};

pub(super) struct NumberView {
    source: Rc<BufferApiView>,
    first: f64,
    origin: usize,
    length: usize,
    x: usize,
    y: usize,
    base: usize,
    first_columns: Option<Vec<f64>>,
    warnings: RefCell<Vec<(f64, f64)>>,
}
impl NumberView {
    pub fn new(
        source: Rc<BufferApiView>,
        first: f64,
        origin: usize,
        rows: usize,
        first_columns: Option<Vec<f64>>,
    ) -> Result<Self, SerializeError> {
        let remaining = source.len()? as f64 - first;
        let tail = if remaining <= rows as f64 {
            remaining.max(0.0).ceil() as usize
        } else {
            rows.saturating_add(1)
        };
        Ok(Self {
            x: source.cursor_x()?,
            y: source.cursor_y()?,
            base: source.base_y()?,
            source,
            first,
            origin,
            length: origin.saturating_add(tail),
            first_columns,
            warnings: RefCell::new(Vec::new()),
        })
    }
    fn actual_row(&self, row: usize) -> f64 {
        self.first + row as f64 - self.origin as f64
    }
    pub fn line(&self, row: usize) -> Result<Option<LineRef>, SerializeError> {
        self.source
            .get_line_number(self.actual_row(row))
            .map_err(Into::into)
    }
    pub fn warnings(&self) -> Vec<(f64, f64)> {
        self.warnings.borrow().clone()
    }
    pub fn columns(start: f64, end: f64) -> Result<Vec<f64>, JsValue> {
        if start.is_nan() || start >= end {
            return Ok(Vec::new());
        }
        if !start.is_finite() || !end.is_finite() {
            return Err(js_sys::RangeError::new("Non-finite serialization traversal").into());
        }
        let mut values = Vec::new();
        let mut value = start;
        while value < end {
            values.push(value);
            let next = value + 1.0;
            if next == value {
                return Err(js_sys::RangeError::new("Serialization index does not advance").into());
            }
            value = next;
        }
        Ok(values)
    }
}
impl BufferAccess for NumberView {
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
        Ok(Some(if row == self.origin {
            self.first_columns.as_ref().map_or(line.len(), Vec::len)
        } else {
            line.len()
        }))
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        self.numeric_cell(row, col)
            .map(|cell| cell.map(|cell| cell.normalized()))
    }
    fn reference_cell(&self, col: usize) -> Result<Option<CellData>, SerializeError> {
        let Some(line) = self.source.get_line_number(0.0)? else {
            return Ok(None);
        };
        let line = line
            .try_borrow()
            .map_err(|_| SerializeError::BorrowedLine)?;
        Ok(line.get_cell(col))
    }
    fn numeric_cell(&self, row: usize, col: usize) -> Result<Option<NumericCell>, SerializeError> {
        let actual = if row == self.origin {
            match &self.first_columns {
                Some(columns) => match columns.get(col) {
                    Some(value) => *value,
                    None => return Ok(None),
                },
                None => col as f64,
            }
        } else {
            col as f64
        };
        let Some(line) = self.line(row)? else {
            return Ok(None);
        };
        let line = line
            .try_borrow()
            .map_err(|_| SerializeError::BorrowedLine)?;
        let cell = get_cell_number(&line, actual);
        if cell.is_none() {
            self.warnings
                .borrow_mut()
                .push((self.actual_row(row), actual));
        }
        Ok(cell)
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
