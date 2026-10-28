// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Conversion between UTF-16 string offsets and physical terminal columns.
use super::{SearchBuffer, SearchError};
use crate::buffer::BufferLine;

pub(super) fn string_length_to_buffer_size(
    buffer: SearchBuffer<'_>,
    row: isize,
    offset: usize,
) -> Result<isize, SearchError> {
    let mut offset = isize::try_from(offset).map_err(|_| SearchError::IndexOverflow)?;
    let Some(line) = buffer.line(row) else {
        return Ok(0);
    };
    let mut i = 0;
    while (i as isize) < offset {
        if line.get_width(i).is_none() {
            break;
        }
        let length = cell_utf16_length(line, i).ok_or(SearchError::InvalidCellData)?;
        if length > 1 {
            offset -= (length - 1) as isize;
        }
        if line.get_width(i + 1) == Some(0) {
            offset = offset.checked_add(1).ok_or(SearchError::IndexOverflow)?;
        }
        i += 1;
    }
    Ok(offset)
}

pub(super) fn buffer_cols_to_string_offset(
    buffer: SearchBuffer<'_>,
    mut row: isize,
    mut cols: isize,
) -> Result<usize, SearchError> {
    let mut offset = 0_usize;
    while cols > 0 {
        let Some(line) = buffer.line(row) else {
            break;
        };
        for i in 0..(cols as usize).min(buffer.cols()) {
            let Some(width) = line.get_width(i) else {
                break;
            };
            if width != 0 {
                let length = if line.get_code_point(i) == Some(0) {
                    1
                } else {
                    cell_utf16_length(line, i).ok_or(SearchError::InvalidCellData)?
                };
                offset = offset
                    .checked_add(length)
                    .ok_or(SearchError::IndexOverflow)?;
            }
        }
        row = row.checked_add(1).ok_or(SearchError::IndexOverflow)?;
        if buffer.line(row).is_some_and(|line| !line.is_wrapped) {
            break;
        }
        cols -= buffer.cols() as isize;
    }
    Ok(offset)
}

fn cell_utf16_length(line: &BufferLine, col: usize) -> Option<usize> {
    if line.is_combined(col)? != 0 {
        return line.get_string(col).map(|units| units.len());
    }
    // Packed single cells need no string allocation or attribute clone merely
    // to count UTF-16 units. Combined strings retain the existing buffer API.
    Some(match line.get_code_point(col)? {
        0 => 0,
        1..=0xffff => 1,
        _ => 2,
    })
}
