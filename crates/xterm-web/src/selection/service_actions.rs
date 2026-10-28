// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Selection commands and word/line range updates.
use super::*;

impl SelectionService {
    pub fn clear_selection(
        &self,
        buffer: &xterm_core::buffer::BufferService,
        dimensions: Dimensions,
    ) -> Result<(), SelectionError> {
        self.model.borrow_mut().clear_selection();
        self.drag_stop.fire(&())?;
        self.refresh(buffer, dimensions, false)?;
        self.selection_change.fire(&())?;
        Ok(())
    }
    pub fn disable(
        &self,
        buffer: &xterm_core::buffer::BufferService,
        dimensions: Dimensions,
    ) -> Result<(), SelectionError> {
        self.clear_selection(buffer, dimensions)?;
        self.enabled.set(false);
        Ok(())
    }
    pub fn select_all(
        &self,
        buffer: &xterm_core::buffer::BufferService,
        dimensions: Dimensions,
    ) -> Result<(), SelectionError> {
        self.model.borrow_mut().is_select_all_active = true;
        self.refresh(buffer, dimensions, false)?;
        self.selection_change.fire(&())?;
        Ok(())
    }
    pub fn select_lines(
        &self,
        buffer: &xterm_core::buffer::BufferService,
        dimensions: Dimensions,
        start: i64,
        end: i64,
    ) -> Result<(), SelectionError> {
        let last = {
            let active = buffer.buffer();
            let active = active.try_borrow().map_err(|_| SelectionError::Borrowed)?;
            i64::try_from(active.lines.len()).map_err(|_| SelectionError::Arithmetic)? - 1
        };
        *self.model.borrow_mut() = SelectionModel {
            selection_start: Some(Position::new(0, start.max(0))),
            selection_end: Some(Position::new(
                i64::from(dimensions.columns.get()),
                end.min(last),
            )),
            ..SelectionModel::default()
        };
        self.refresh(buffer, dimensions, false)?;
        self.selection_change.fire(&())?;
        Ok(())
    }
    pub fn set_selection(
        &self,
        buffer: &xterm_core::buffer::BufferService,
        dimensions: Dimensions,
        start: Position,
        length: i64,
    ) -> Result<(), SelectionError> {
        *self.model.borrow_mut() = SelectionModel {
            selection_start: Some(start),
            selection_start_length: length,
            ..SelectionModel::default()
        };
        self.drag_stop.fire(&())?;
        self.refresh(buffer, dimensions, false)?;
        self.fire_if_changed(dimensions)
    }
    pub fn handle_trim(
        &self,
        buffer: &xterm_core::buffer::BufferService,
        dimensions: Dimensions,
        amount: u32,
    ) -> Result<(), SelectionError> {
        let refresh = self.model.borrow_mut().handle_trim(amount)?;
        if refresh {
            self.refresh(buffer, dimensions, false)?;
        }
        Ok(())
    }
    pub fn select_line_at(
        &self,
        buffer: &Buffer,
        dimensions: Dimensions,
        row: usize,
    ) -> Result<(), SelectionError> {
        let range = buffer.get_wrapped_range_for_line(row)?;
        let rows = range
            .last
            .checked_sub(range.first)
            .and_then(|rows| rows.checked_add(1))
            .ok_or(SelectionError::Arithmetic)?;
        let length = u32::try_from(rows)
            .ok()
            .and_then(|rows| rows.checked_mul(dimensions.columns.get()))
            .ok_or(SelectionError::Arithmetic)?;
        let row = i64::try_from(range.first).map_err(|_| SelectionError::Arithmetic)?;
        let mut model = self.model.borrow_mut();
        model.selection_start = Some(Position::new(0, row));
        model.selection_end = None;
        model.selection_start_length = i64::from(length);
        Ok(())
    }
    pub fn select_word_at(
        &self,
        buffer: &Buffer,
        dimensions: Dimensions,
        position: Position,
        separators: &[u16],
        allow_whitespace: bool,
    ) -> Result<(), SelectionError> {
        if let Some(word) = word::get_word_at(
            buffer,
            position,
            dimensions.columns.get(),
            separators,
            allow_whitespace,
        )? {
            let start = normalized_word_start(position.row, word.start, dimensions.columns.get())?;
            let mut model = self.model.borrow_mut();
            model.selection_start = Some(start);
            model.selection_start_length = i64::from(word.length);
        }
        Ok(())
    }
    pub fn select_to_word_at(
        &self,
        buffer: &Buffer,
        dimensions: Dimensions,
        position: Position,
        separators: &[u16],
    ) -> Result<(), SelectionError> {
        if let Some(word) =
            word::get_word_at(buffer, position, dimensions.columns.get(), separators, true)?
        {
            let mut end =
                normalized_word_start(position.row, word.start, dimensions.columns.get())?;
            let mut model = self.model.borrow_mut();
            if !model.are_selection_values_reversed() {
                let sum = end
                    .column
                    .checked_add(i64::from(word.length))
                    .ok_or(SelectionError::Arithmetic)?;
                let columns = i64::from(dimensions.columns.get());
                let offset = if sum > columns {
                    (sum - 1) / columns
                } else {
                    0
                };
                end.column = sum - offset * columns;
                end.row = end
                    .row
                    .checked_add(offset)
                    .ok_or(SelectionError::Arithmetic)?;
            }
            model.selection_end = Some(end);
        }
        Ok(())
    }
}
