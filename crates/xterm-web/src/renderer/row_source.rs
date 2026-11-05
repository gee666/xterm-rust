// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Shared row planning reads cells lazily. Browser ranges remain host values.
use super::{row::RowContext, row_types::*};
use xterm_core::buffer::BufferLine;

pub(super) trait RowSource {
    fn length(&self) -> usize;
    fn cell(&self, x: f64) -> RenderCell;
    fn selected(&self, x: f64, row: usize) -> bool;
    fn valid<E>(&self, x: f64) -> Result<bool, RowError<E>>;
    fn join<E>(
        &mut self,
        x: f64,
        cell: &mut RenderCell,
        valid: &mut bool,
        options: &RowOptions,
        cursor_x: f64,
    ) -> Result<Option<f64>, RowError<E>>;
}
pub(super) struct TypedRow<'a> {
    pub line: &'a BufferLine,
    pub ranges: &'a [[usize; 2]],
    pub selection: Selection,
    pub skip: f64,
}
impl RowSource for TypedRow<'_> {
    fn length(&self) -> usize {
        self.line.get_no_bg_trimmed_length()
    }
    fn cell(&self, x: f64) -> RenderCell {
        RenderCell::from(self.line.get_cell(x as usize).unwrap_or_default())
    }
    fn selected(&self, x: f64, row: usize) -> bool {
        self.selection.contains(x as usize, row)
    }
    fn valid<E>(&self, x: f64) -> Result<bool, RowError<E>> {
        Ok(x >= self.skip)
    }
    fn join<E>(
        &mut self,
        x: f64,
        cell: &mut RenderCell,
        valid: &mut bool,
        options: &RowOptions,
        cursor_x: f64,
    ) -> Result<Option<f64>, RowError<E>> {
        let Some(range) = self
            .ranges
            .first()
            .copied()
            .filter(|r| r[0] as f64 == x && *valid)
        else {
            return Ok(None);
        };
        self.ranges = &self.ranges[1..];
        if range[1] <= range[0] || range[1] > self.line.len() {
            return Err(RowError::JoinRange(range));
        }
        let selected = self.selected(range[0] as f64, options.row);
        *valid =
            (range[0] + 1..range[1]).all(|col| self.selected(col as f64, options.row) == selected);
        *valid &= !options.cursor_row || cursor_x < range[0] as f64 || cursor_x >= range[1] as f64;
        if !*valid {
            self.skip = range[1] as f64;
            return Ok(None);
        }
        cell.join(
            self.line
                .translate_to_string(true, Some(range[0]), Some(range[1]), None),
            (range[1] - range[0]) as f64,
        );
        Ok(Some((range[1] - 1) as f64))
    }
}
impl<'a> TypedRow<'a> {
    pub fn new(line: &'a BufferLine, context: &RowContext<'a>, selection: Selection) -> Self {
        Self {
            line,
            ranges: context.joined_ranges,
            selection,
            skip: 0.0,
        }
    }
}
