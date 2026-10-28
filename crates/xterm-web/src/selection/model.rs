// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of browser/selection/SelectionModel.ts.

use std::{error::Error, fmt, num::NonZeroU32};

/// A buffer cell boundary. Search inside combined cells can produce signed columns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Position {
    pub column: i64,
    pub row: i64,
}

impl Position {
    pub const fn new(column: i64, row: i64) -> Self {
        Self { column, row }
    }
}

/// Live buffer dimensions, supplied on each query so resize/select-all cannot go stale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Dimensions {
    pub columns: NonZeroU32,
    pub rows: NonZeroU32,
    pub scrollback: u32,
}

/// Coordinate arithmetic cannot be represented by the Rust integer type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CoordinateOverflow;

impl fmt::Display for CoordinateOverflow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("selection exceeds the signed 64-bit coordinate range")
    }
}

impl Error for CoordinateOverflow {}

/// Selection coordinates only, independent of cell contents and browser events.
///
/// Columns and lengths remain signed because addon searches inside combined
/// cells can produce negative values. Arithmetic is checked; browser bindings
/// validate JavaScript integer ranges before constructing native coordinates.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SelectionModel {
    pub is_select_all_active: bool,
    pub selection_start_length: i64,
    pub selection_start: Option<Position>,
    pub selection_end: Option<Position>,
}

impl SelectionModel {
    pub fn clear_selection(&mut self) {
        *self = Self::default();
    }

    pub fn are_selection_values_reversed(&self) -> bool {
        match (self.selection_start, self.selection_end) {
            (Some(start), Some(end)) => (start.row, start.column) > (end.row, end.column),
            _ => false,
        }
    }

    pub fn final_selection_start(&self) -> Option<Position> {
        if self.is_select_all_active {
            return Some(Position::default());
        }
        if self.are_selection_values_reversed() {
            self.selection_end
        } else {
            self.selection_start
        }
    }

    pub fn final_selection_end(
        &self,
        dimensions: Dimensions,
    ) -> Result<Option<Position>, CoordinateOverflow> {
        if self.is_select_all_active {
            return Ok(Some(Position::new(
                i64::from(dimensions.columns.get()),
                i64::from(dimensions.scrollback) + i64::from(dimensions.rows.get()) - 1,
            )));
        }
        let Some(start) = self.selection_start else {
            return Ok(None);
        };
        let columns = i64::from(dimensions.columns.get());
        let start_plus_length = || {
            start
                .column
                .checked_add(self.selection_start_length)
                .ok_or(CoordinateOverflow)
        };
        let end = match self.selection_end {
            Some(end) if !self.are_selection_values_reversed() => end,
            _ => return Self::minimum_end(start, columns, start_plus_length()?, true).map(Some),
        };
        if self.selection_start_length != 0 && end.row == start.row {
            let start_plus_length = start_plus_length()?;
            if start_plus_length > columns {
                // Unlike the absent/reversed-end branch, upstream retains the
                // next row's column zero for an exact wrapped word here.
                return Self::minimum_end(start, columns, start_plus_length, false).map(Some);
            }
            return Ok(Some(Position::new(
                start_plus_length.max(end.column),
                end.row,
            )));
        }
        Ok(Some(end))
    }

    fn minimum_end(
        start: Position,
        columns: i64,
        start_plus_length: i64,
        exclude_trailing_eol: bool,
    ) -> Result<Position, CoordinateOverflow> {
        if start_plus_length <= columns {
            return Ok(Position::new(start_plus_length, start.row));
        }
        let remainder = start_plus_length % columns;
        let mut row_offset = start_plus_length / columns;
        let column = if remainder == 0 && exclude_trailing_eol {
            row_offset -= 1;
            columns
        } else {
            remainder
        };
        // This branch only divides a positive value by a positive column count.
        let row = start
            .row
            .checked_add(row_offset)
            .ok_or(CoordinateOverflow)?;
        Ok(Position::new(column, row))
    }

    /// Returns whether clearing or clamping requires a selection refresh.
    /// On numeric overflow the original selection is left unchanged.
    pub fn handle_trim(&mut self, amount: u32) -> Result<bool, CoordinateOverflow> {
        let trim = |point: Position| {
            point
                .row
                .checked_sub(i64::from(amount))
                .map(|row| Position::new(point.column, row))
                .ok_or(CoordinateOverflow)
        };
        let start = self.selection_start.map(trim).transpose()?;
        let end = self.selection_end.map(trim).transpose()?;
        self.selection_start = start;
        self.selection_end = end;
        if end.is_some_and(|point| point.row < 0) {
            self.clear_selection();
            return Ok(true);
        }
        if start.is_some_and(|point| point.row < 0) {
            self.selection_start = Some(Position::default());
            return Ok(true);
        }
        Ok(false)
    }
}
