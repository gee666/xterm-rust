// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Selection text extraction uses buffer translation, preserving UTF-16 units.
use super::{
    model::{Dimensions, SelectionModel},
    service::{SelectionError, SelectionMode},
};
use xterm_core::buffer::Buffer;

fn translate_signed(
    buffer: &Buffer,
    row: i64,
    start: Option<i64>,
    end: Option<i64>,
) -> Result<Vec<u16>, SelectionError> {
    let Ok(row_index) = usize::try_from(row) else {
        return Ok(Vec::new());
    };
    if start.unwrap_or(0) >= 0 && end.is_none_or(|end| end >= 0) {
        return buffer
            .translate_buffer_line_to_string(
                row_index,
                true,
                start
                    .map(usize::try_from)
                    .transpose()
                    .map_err(|_| SelectionError::Arithmetic)?,
                end.map(usize::try_from)
                    .transpose()
                    .map_err(|_| SelectionError::Arithmetic)?,
            )
            .map_err(SelectionError::from);
    }
    let Some(line) = usize::try_from(row)
        .ok()
        .and_then(|row| buffer.lines.get(row))
    else {
        return Ok(Vec::new());
    };
    let line = line.try_borrow().map_err(|_| SelectionError::Borrowed)?;
    let start = start.unwrap_or(0);
    let end = end
        .unwrap_or(i64::try_from(line.len()).map_err(|_| SelectionError::Arithmetic)?)
        .min(i64::try_from(line.get_trimmed_length()).map_err(|_| SelectionError::Arithmetic)?);
    if start >= end {
        return Ok(Vec::new());
    }
    // JavaScript typed-array reads at negative columns yield undefined. Its
    // bitwise operations turn that into a blank cell, advancing by one column.
    let prefix = if start < 0 {
        end.min(0)
            .checked_sub(start)
            .ok_or(SelectionError::Arithmetic)?
    } else {
        0
    };
    let prefix = usize::try_from(prefix).map_err(|_| SelectionError::Arithmetic)?;
    let mut result = Vec::new();
    result
        .try_reserve_exact(prefix)
        .map_err(SelectionError::Allocation)?;
    result.resize(prefix, 32);
    if end > 0 {
        result.extend(line.translate_to_string(
            true,
            Some(usize::try_from(start.max(0)).map_err(|_| SelectionError::Arithmetic)?),
            Some(usize::try_from(end).map_err(|_| SelectionError::Arithmetic)?),
            None,
        ));
    }
    Ok(result)
}

pub fn selection_text(
    buffer: &Buffer,
    model: &SelectionModel,
    dimensions: Dimensions,
    mode: SelectionMode,
    windows_newlines: bool,
) -> Result<Vec<u16>, SelectionError> {
    let (Some(start), Some(end)) = (
        model.final_selection_start(),
        model.final_selection_end(dimensions)?,
    ) else {
        return Ok(Vec::new());
    };
    let mut result: Vec<Vec<u16>> = Vec::new();
    let translate =
        |row: i64, start: Option<i64>, end: Option<i64>| translate_signed(buffer, row, start, end);
    let wrapped = |row: i64| -> Result<bool, SelectionError> {
        let Some(line) = usize::try_from(row)
            .ok()
            .and_then(|row| buffer.lines.get(row))
        else {
            return Ok(false);
        };
        let wrapped = line
            .try_borrow()
            .map_err(|_| SelectionError::Borrowed)?
            .is_wrapped;
        Ok(wrapped)
    };
    if mode == SelectionMode::Column {
        if start.column == end.column {
            return Ok(Vec::new());
        }
        let left = start.column.min(end.column);
        let right = start.column.max(end.column);
        for row in start.row..=end.row {
            result.push(translate(row, Some(left), Some(right))?);
        }
    } else {
        result.push(translate(
            start.row,
            Some(start.column),
            (start.row == end.row).then_some(end.column),
        )?);
        if start.row < end.row {
            let first_middle = start.row.checked_add(1).ok_or(SelectionError::Arithmetic)?;
            for row in first_middle..=end.row {
                let text = translate(row, None, (row == end.row).then_some(end.column))?;
                if wrapped(row)? {
                    // The first row is always pushed before wrapped continuation.
                    if let Some(previous) = result.last_mut() {
                        previous.extend(text);
                    }
                } else {
                    result.push(text);
                }
            }
        }
    }
    let mut joined = Vec::new();
    for (index, row) in result.into_iter().enumerate() {
        if index != 0 {
            if windows_newlines {
                joined.push(13);
            }
            joined.push(10);
        }
        joined.extend(
            row.into_iter()
                .map(|unit| if unit == 0xa0 { 32 } else { unit }),
        );
    }
    Ok(joined)
}
