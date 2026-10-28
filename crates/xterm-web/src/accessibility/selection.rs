// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Accessibility DOM UTF-16 to terminal cell coordinates.
use std::fmt;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellPoint {
    pub row: f64,
    pub column: f64,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SelectionRange {
    pub row: f64,
    pub column: f64,
    pub length: f64,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidRange;
impl fmt::Display for InvalidRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid range")
    }
}
impl std::error::Error for InvalidRange {}

/// `row` is the zero-based aria-posinset conversion. NaN means a stale or
/// externally edited row, which upstream warns about and ignores. Columns are
/// indexed in UTF-16 units, including both units of astral characters.
pub fn map_point(row: f64, offset: usize, columns: &[usize], cols: usize) -> Option<CellPoint> {
    let column = columns.get(offset).map_or_else(
        || columns.last().map_or(f64::NAN, |value| *value as f64 + 1.0),
        |value| *value as f64,
    );
    normalize_point(row, column, cols)
}

pub(super) fn normalize_point(row: f64, column: f64, cols: usize) -> Option<CellPoint> {
    if row.is_nan() {
        return None;
    }
    let mut row = row;
    let mut column = column;
    if column >= cols as f64 {
        row += 1.0;
        column = 0.0;
    }
    Some(CellPoint { row, column })
}

/// Match source comparisons before passing the original numeric values to the
/// terminal selection API. Do not silently normalize or clamp a reversed range.
pub fn selection_range(
    start: CellPoint,
    end: CellPoint,
    cols: usize,
) -> Result<SelectionRange, InvalidRange> {
    if start.row > end.row || start.row == end.row && start.column >= end.column {
        return Err(InvalidRange);
    }
    Ok(SelectionRange {
        row: start.row,
        column: start.column,
        length: (end.row - start.row) * cols as f64 - start.column + end.column,
    })
}
