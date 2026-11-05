// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{CellRange, SerializeError};

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
    pub index: Option<usize>,
    pub start_col: usize,
    pub end_col: Option<usize>,
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
                    index: Some(row),
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
                // Public getLine does not find negative, fractional or
                // out-of-address-space array keys. They still emit an HTML row.
                let index = if row >= 0.0 && row.fract() == 0.0 && row < usize::MAX as f64 {
                    Some(row as usize)
                } else {
                    None
                };
                Some(Row {
                    index,
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
