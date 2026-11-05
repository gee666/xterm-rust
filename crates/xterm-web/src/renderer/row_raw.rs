// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Raw DOM joiners must not be projected into integers before the row reads them.
use super::{
    row::{RenderedRow, RowContext, RowFactory},
    row_operators::*,
    row_source::RowSource,
    row_types::*,
    width_cache::{WidthCache, WidthMeasurer},
};
use crate::character_joiner::{raw::RawRanges, LineRef};
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use xterm_core::buffer::CellData;
use xterm_core::public_api::line_numbers::read_cell_number;

struct RawRow<'a> {
    line: &'a LineRef,
    work_cell: &'a RefCell<CellData>,
    ranges: RawRanges,
    skip: JsValue,
    selection: &'a dyn Fn() -> Selection,
}
impl RowFactory {
    /// Consume the live outer array with shift, preserving inner getters and
    /// errors. No line/selection RefCell borrow survives a JavaScript operation.
    pub fn create_row_raw<M: WidthMeasurer>(
        &self,
        line: &LineRef,
        work_cell: &RefCell<CellData>,
        ranges: RawRanges,
        selection: &dyn Fn() -> Selection,
        context: RowContext<'_>,
        widths: &WidthCache<M>,
    ) -> Result<RenderedRow, RowError<M::Error>> {
        self.plan_row(
            RawRow {
                line,
                work_cell,
                ranges,
                skip: 0.into(),
                selection,
            },
            context,
            widths,
        )
    }
}
impl RowSource for RawRow<'_> {
    fn length(&self) -> usize {
        self.line.borrow().get_no_bg_trimmed_length()
    }
    fn cell(&self, x: f64) -> RenderCell {
        let cell = read_cell_number(&self.line.borrow(), x).normalized();
        *self.work_cell.borrow_mut() = cell.clone();
        RenderCell::from(cell)
    }
    fn selected(&self, x: f64, row: usize) -> bool {
        (self.selection)().contains_number(x, row)
    }
    fn valid<E>(&self, x: f64) -> Result<bool, RowError<E>> {
        le(&self.skip, &x.into()).map_err(RowError::Browser)
    }
    fn join<E>(
        &mut self,
        x: f64,
        cell: &mut RenderCell,
        valid: &mut bool,
        options: &RowOptions,
        cursor_x: f64,
    ) -> Result<Option<f64>, RowError<E>> {
        self.join_inner(x, cell, valid, options, cursor_x)
            .map_err(RowError::Browser)
    }
}
impl RawRow<'_> {
    // Source has one reusable work cell per factory. A recursive createRow can
    // replace its contents without changing the width captured by the outer row.
    fn sync_cell(&self, cell: &mut RenderCell) {
        let width = cell.width;
        *cell = RenderCell::from(self.work_cell.borrow().clone());
        cell.width = width;
    }
    fn selected_raw(&self, x: &JsValue, row: usize) -> Result<bool, JsValue> {
        let Selection {
            start: Some(start),
            end: Some(end),
            column,
        } = (self.selection)()
        else {
            return Ok(false);
        };
        let y = row as i128;
        let sy = i128::from(start[1]);
        let ey = i128::from(end[1]);
        if column {
            if start[0] <= end[0] {
                return Ok(le(&(start[0] as f64).into(), x)?
                    && y >= sy
                    && less(x, &(end[0] as f64).into())?
                    && y <= ey);
            }
            return Ok(less(x, &(start[0] as f64).into())?
                && y >= sy
                && le(&(end[0] as f64).into(), x)?
                && y <= ey);
        }
        Ok((y > sy && y < ey)
            || (sy == ey
                && y == sy
                && le(&(start[0] as f64).into(), x)?
                && less(x, &(end[0] as f64).into())?)
            || (sy < ey && y == ey && less(x, &(end[0] as f64).into())?)
            || (sy < ey && y == sy && le(&(start[0] as f64).into(), x)?))
    }
    fn join_inner(
        &mut self,
        x: f64,
        cell: &mut RenderCell,
        valid: &mut bool,
        options: &RowOptions,
        cursor_x: f64,
    ) -> Result<Option<f64>, JsValue> {
        // The start getter runs even when the skip guard is false.
        if !less(&0.into(), &get(&self.ranges.0, &"length".into())?)?
            || at(&at(&self.ranges.0, 0.0)?, 0.0)?.as_f64() != Some(x)
            || !*valid
        {
            self.sync_cell(cell);
            return Ok(None);
        }
        let range = shift(&self.ranges.0)?;
        let first = self.selected_raw(&at(&range, 0.0)?, options.row)?;
        let mut i = add(&at(&range, 0.0)?, &1.into())?;
        loop {
            let end = at(&range, 1.0)?;
            if !less(&i, &end)? {
                break;
            }
            // The source loops forever for a literal +Infinity endpoint. Do not
            // freeze the Rust renderer. This safety divergence is documented.
            if end.as_f64() == Some(f64::INFINITY) {
                return Err(
                    js_sys::RangeError::new("joined selection range cannot terminate").into(),
                );
            }
            if *valid {
                *valid = first == self.selected_raw(&i, options.row)?;
            }
            let next = increment(&i)?;
            if next.as_f64().is_some_and(|v| Some(v) == i.as_f64()) {
                return Err(
                    js_sys::RangeError::new("joined selection column cannot advance").into(),
                );
            }
            i = next;
        }
        if *valid && options.cursor_row {
            *valid = less(&cursor_x.into(), &at(&range, 0.0)?)?
                || le(&at(&range, 1.0)?, &cursor_x.into())?;
        }
        if !*valid {
            self.skip = at(&range, 1.0)?;
            self.sync_cell(cell);
            return Ok(None);
        }
        let chars = self.translate(at(&range, 0.0)?, at(&range, 1.0)?)?;
        let width = difference(&at(&range, 1.0)?, &at(&range, 0.0)?)?;
        *cell = RenderCell::from(self.work_cell.borrow().clone());
        cell.join(chars, width.as_f64().unwrap_or(f64::NAN));
        cell.raw_width = Some(width);
        Ok(Some(subtract_one(&at(&range, 1.0)?)?))
    }
    fn translate(&self, mut start: JsValue, mut end: JsValue) -> Result<Vec<u16>, JsValue> {
        if start.is_null() || start.is_undefined() {
            start = 0.into();
        }
        if end.is_null() || end.is_undefined() {
            end = (self.line.borrow().len() as f64).into();
        }
        let trimmed = self.line.borrow().get_trimmed_length() as f64;
        let end = min(&end, trimmed)?;
        let mut result = Vec::new();
        while less(&start, &end)? {
            let index = address(&start)? / 3.0;
            let cell = read_cell_number(&self.line.borrow(), index);
            if let Some(chars) = cell.chars() {
                if chars.is_empty()
                    && cell.content.unwrap_or(0)
                        & xterm_core::buffer::constants::content::IS_COMBINED_MASK
                        == 0
                {
                    result.push(32);
                } else {
                    result.extend(chars);
                }
            }
            let width = cell.metadata().get_width();
            let next = add(&start, &if width == 0 { 1 } else { width }.into())?;
            if next.as_f64().is_some_and(|v| Some(v) == start.as_f64()) {
                return Err(js_sys::RangeError::new("joined text column cannot advance").into());
            }
            start = next;
        }
        Ok(result)
    }
}
