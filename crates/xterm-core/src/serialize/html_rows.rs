// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{BufferAccess, CellRange, SerializeError};
use crate::buffer::CellData;

/// HTML row coordinates retain JavaScript numeric values rather than truncating
/// them to buffer indexes. Columns remain integral and the end is exclusive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HtmlNumericRange {
    pub start_line: f64,
    pub end_line: f64,
    pub start_col: usize,
    pub end_col: usize,
}

pub(super) struct Row {
    index: Index,
    pub start_col: usize,
    pub end_col: Option<usize>,
}
enum Index {
    Integral(usize),
    Numeric(f64),
}
impl Row {
    pub(super) fn capture(
        &self,
        buffer: &dyn BufferAccess,
    ) -> Result<Option<usize>, SerializeError> {
        match self.index {
            Index::Integral(row) => buffer.capture_line(row),
            Index::Numeric(row) => buffer.capture_numeric_line(row),
        }
    }
    pub(super) fn cell(
        &self,
        buffer: &dyn BufferAccess,
        col: usize,
    ) -> Result<Option<CellData>, SerializeError> {
        match self.index {
            Index::Integral(row) => buffer.captured_cell(row, col),
            Index::Numeric(row) => buffer.captured_numeric_cell(row, col),
        }
    }
    pub(super) fn missing_cell(
        &self,
        buffer: &dyn BufferAccess,
        col: usize,
    ) -> Result<(), SerializeError> {
        match self.index {
            Index::Integral(row) => buffer.missing_cell(row, col),
            Index::Numeric(row) => buffer.missing_numeric_cell(row, col),
        }
    }
}

pub(super) enum Rows {
    Integral {
        range: CellRange,
        rows: std::ops::RangeInclusive<usize>,
    },
    Numeric {
        range: HtmlNumericRange,
        next: f64,
    },
}
impl Rows {
    pub(super) fn integral(range: CellRange) -> Self {
        Self::Integral {
            range,
            rows: range.start.y..=range.end.y,
        }
    }
    pub(super) fn numeric(range: HtmlNumericRange) -> Result<Self, SerializeError> {
        // Outside the exact-integer domain, row++ can stop advancing. Do not
        // reproduce a source hang or pretend those inputs are compatible.
        const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
        if !range.start_line.is_finite()
            || !range.end_line.is_finite()
            || range.start_line.abs() > MAX_SAFE_INTEGER
            || range.end_line.abs() > MAX_SAFE_INTEGER
        {
            return Err(SerializeError::InvalidRange);
        }
        Ok(Self::Numeric {
            next: range.start_line,
            range,
        })
    }
}
impl Iterator for Rows {
    type Item = Row;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Integral { range, rows } => {
                let row = rows.next()?;
                Some(Row {
                    index: Index::Integral(row),
                    start_col: if row == range.start.y {
                        range.start.x
                    } else {
                        0
                    },
                    end_col: (row == range.end.y).then_some(range.end.x),
                })
            }
            Self::Numeric { range, next } => {
                let row = *next;
                if row > range.end_line {
                    return None;
                }
                *next += 1.0;
                // Keep the key raw through lookup. CircularList adds its
                // current start index before remainder, which can round even a
                // fractional key into an existing slot.
                Some(Row {
                    index: Index::Numeric(row),
                    start_col: if row == range.start_line {
                        range.start_col
                    } else {
                        0
                    },
                    end_col: (row == range.end_line).then_some(range.end_col),
                })
            }
        }
    }
}
