// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Click/drag state changes without DOM or clock ownership.
use super::*;
use xterm_core::buffer::BufferService;

impl SelectionService {
    pub fn incremental_click(&self, position: Option<Position>) {
        let mut model = self.model.borrow_mut();
        if model.selection_start.is_some() {
            model.selection_end = position;
        }
    }
    pub fn single_click(
        &self,
        buffers: &BufferService,
        dimensions: Dimensions,
        position: Option<Position>,
        column_mode: bool,
    ) -> Result<(), SelectionError> {
        let had_selection = self.has_selection(dimensions)?;
        {
            let mut model = self.model.borrow_mut();
            model.selection_start_length = 0;
            model.is_select_all_active = false;
            self.mode.set(if column_mode {
                SelectionMode::Column
            } else {
                SelectionMode::Normal
            });
            model.selection_start = position;
            if position.is_none() {
                return Ok(());
            }
            model.selection_end = None;
        }
        if had_selection {
            self.previous.set(PreviousSelection {
                start: self.selection_start(),
                end: self.selection_end(dimensions)?,
                has_selection: false,
            });
            self.selection_change.fire(&())?;
        }
        if let Some(mut start) = self.selection_start() {
            let active = buffers.buffer();
            let buffer = active.try_borrow().map_err(|_| SelectionError::Borrowed)?;
            let line = usize::try_from(start.row)
                .ok()
                .and_then(|row| buffer.lines.get(row));
            if let Some(line) = line {
                let line = line.try_borrow().map_err(|_| SelectionError::Borrowed)?;
                let column =
                    usize::try_from(start.column).map_err(|_| SelectionError::Arithmetic)?;
                if column != line.len() && line.has_width(column) == Some(0) {
                    start.column = start
                        .column
                        .checked_add(1)
                        .ok_or(SelectionError::Arithmetic)?;
                    self.model.borrow_mut().selection_start = Some(start);
                }
            }
        }
        Ok(())
    }
    pub fn drag_move(
        &self,
        buffers: &BufferService,
        dimensions: Dimensions,
        position: Option<Position>,
        scroll_amount: f64,
        separators: &[u16],
    ) -> Result<(), SelectionError> {
        if self.model.borrow().selection_start.is_none() {
            return Ok(());
        }
        let previous = self.model.borrow().selection_end;
        self.model.borrow_mut().selection_end = position;
        let Some(mut end) = position else {
            return self.refresh(buffers, dimensions, true);
        };
        let start = self.model.borrow().selection_start;
        if self.mode() == SelectionMode::Line {
            end.column = if start.is_some_and(|start| end.row < start.row) {
                0
            } else {
                i64::from(dimensions.columns.get())
            };
            self.model.borrow_mut().selection_end = Some(end);
        } else if self.mode() == SelectionMode::Word {
            let active = buffers.buffer();
            let buffer = active.try_borrow().map_err(|_| SelectionError::Borrowed)?;
            self.select_to_word_at(&buffer, dimensions, end, separators)?;
            end = self
                .model
                .borrow()
                .selection_end
                .ok_or(SelectionError::Arithmetic)?;
        }
        if self.mode() != SelectionMode::Column {
            if scroll_amount > 0.0 {
                end.column = i64::from(dimensions.columns.get());
            } else if scroll_amount < 0.0 {
                end.column = 0;
            }
        }
        {
            let active = buffers.buffer();
            let buffer = active.try_borrow().map_err(|_| SelectionError::Borrowed)?;
            let line = usize::try_from(end.row)
                .ok()
                .and_then(|row| buffer.lines.get(row));
            if let Some(line) = line {
                let line = line.try_borrow().map_err(|_| SelectionError::Borrowed)?;
                if end.column < i64::from(dimensions.columns.get())
                    && line.has_width(
                        usize::try_from(end.column).map_err(|_| SelectionError::Arithmetic)?,
                    ) == Some(0)
                {
                    end.column += 1;
                }
            }
        }
        self.model.borrow_mut().selection_end = Some(end);
        if previous != Some(end) {
            self.refresh(buffers, dimensions, true)?;
        }
        Ok(())
    }
    /// The host scrolls first, then calls this with the live post-scroll buffer.
    pub fn update_after_drag_scroll(
        &self,
        buffers: &BufferService,
        dimensions: Dimensions,
        amount: f64,
    ) -> Result<(), SelectionError> {
        if amount == 0.0 {
            return Ok(());
        }
        let mut model = self.model();
        if model.selection_start.is_none() {
            return Ok(());
        }
        let Some(mut end) = model.selection_end else {
            return Ok(());
        };
        {
            let active = buffers.buffer();
            let buffer = active.try_borrow().map_err(|_| SelectionError::Borrowed)?;
            let row = if amount > 0.0 {
                buffer
                    .ydisp
                    .saturating_add(
                        usize::try_from(dimensions.rows.get())
                            .map_err(|_| SelectionError::Arithmetic)?
                            - 1,
                    )
                    .min(buffer.lines.len().saturating_sub(1))
            } else {
                buffer.ydisp
            };
            end.row = i64::try_from(row).map_err(|_| SelectionError::Arithmetic)?;
        }
        if self.mode() != SelectionMode::Column {
            end.column = if amount > 0.0 {
                i64::from(dimensions.columns.get())
            } else {
                0
            };
        }
        model.selection_end = Some(end);
        *self.model.borrow_mut() = model;
        self.refresh(buffers, dimensions, false)
    }
}
