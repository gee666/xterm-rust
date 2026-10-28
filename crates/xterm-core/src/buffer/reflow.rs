// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Wrapped-row layout helpers adapted from common/buffer/BufferReflow.ts.
//! Rows must be resized to the destination width before widening copies cells.
use super::{BufferLine, CellData};
use crate::{
    circular_list::{CircularList, CircularListError, ListChange},
    event::EmitError,
};
use std::{cell::RefCell, fmt, rc::Rc};

type LineRef = Rc<RefCell<BufferLine>>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ReflowError {
    InvalidColumns,
    InvalidRange,
    MissingLine,
    BorrowedLine,
    Dispatch(EmitError),
    Storage(CircularListError),
}
impl fmt::Display for ReflowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidColumns => f.write_str("invalid reflow column count"),
            Self::InvalidRange => f.write_str("invalid reflow layout range"),
            Self::MissingLine => f.write_str("missing reflow buffer line"),
            Self::BorrowedLine => f.write_str("reflow buffer line is already borrowed"),
            Self::Dispatch(error) => error.fmt(f),
            Self::Storage(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for ReflowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dispatch(error) => Some(error),
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}
impl From<CircularListError> for ReflowError {
    fn from(error: CircularListError) -> Self {
        Self::Storage(error)
    }
}
impl From<EmitError> for ReflowError {
    fn from(error: EmitError) -> Self {
        Self::Dispatch(error)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NewLayoutResult {
    pub layout: Vec<usize>,
    pub count_removed: usize,
}

/// Last rows use actual content; intermediate rows omit the wide-wrap gap only.
pub fn get_wrapped_line_trimmed_length(
    lines: &[LineRef],
    i: usize,
    cols: usize,
) -> Result<usize, ReflowError> {
    if cols == 0 {
        return Err(ReflowError::InvalidColumns);
    }
    let line = lines
        .get(i)
        .ok_or(ReflowError::MissingLine)?
        .try_borrow()
        .map_err(|_| ReflowError::BorrowedLine)?;
    if i == lines.len() - 1 {
        return Ok(line.get_trimmed_length());
    }
    let ends_in_null = line.has_content(cols - 1) == Some(0) && line.get_width(cols - 1) == Some(1);
    let starts_wide = lines[i + 1]
        .try_borrow()
        .map_err(|_| ReflowError::BorrowedLine)?
        .get_width(0)
        == Some(2);
    Ok(if ends_in_null && starts_wide {
        cols - 1
    } else {
        cols
    })
}

/// Return source-row removal ranges while compacting the surviving wrapped rows.
/// Cursor exclusion applies to the whole wrapped block, including its first row.
pub fn reflow_larger_get_lines_to_remove(
    lines: &CircularList<LineRef>,
    old_cols: usize,
    new_cols: usize,
    buffer_absolute_y: usize,
    null_cell: &CellData,
    reflow_cursor_line: bool,
) -> Result<Vec<(usize, usize)>, ReflowError> {
    if old_cols == 0 || new_cols <= old_cols {
        return Err(ReflowError::InvalidColumns);
    }
    let mut to_remove = Vec::new();
    let mut y = 0;
    while y + 1 < lines.len() {
        let mut i = y + 1;
        if !lines
            .get(i)
            .ok_or(ReflowError::MissingLine)?
            .try_borrow()
            .map_err(|_| ReflowError::BorrowedLine)?
            .is_wrapped
        {
            y += 1;
            continue;
        }
        let mut wrapped = vec![lines.get(y).ok_or(ReflowError::MissingLine)?];
        while i < lines.len() {
            let next = lines.get(i).ok_or(ReflowError::MissingLine)?;
            if !next
                .try_borrow()
                .map_err(|_| ReflowError::BorrowedLine)?
                .is_wrapped
            {
                break;
            }
            wrapped.push(next);
            i += 1;
        }
        if reflow_cursor_line || !(y..i).contains(&buffer_absolute_y) {
            let (dest_line, dest_col) = compact(&wrapped, old_cols, new_cols, null_cell)?;
            wrapped
                .get(dest_line)
                .ok_or(ReflowError::InvalidRange)?
                .try_borrow_mut()
                .map_err(|_| ReflowError::BorrowedLine)?
                .replace_cells(dest_col, new_cols, null_cell, false);
            let mut count = 0;
            for row in (1..wrapped.len()).rev() {
                if row > dest_line
                    || wrapped[row]
                        .try_borrow()
                        .map_err(|_| ReflowError::BorrowedLine)?
                        .get_trimmed_length()
                        == 0
                {
                    count += 1;
                } else {
                    break;
                }
            }
            if count > 0 {
                to_remove.push((y + wrapped.len() - count, count));
            }
        }
        y += wrapped.len();
    }
    Ok(to_remove)
}

fn copy_cells(
    lines: &[LineRef],
    source: usize,
    from: usize,
    dest: usize,
    to: usize,
    count: usize,
) -> Result<(), ReflowError> {
    let source = lines.get(source).ok_or(ReflowError::InvalidRange)?;
    let dest = lines.get(dest).ok_or(ReflowError::InvalidRange)?;
    let mut target = dest
        .try_borrow_mut()
        .map_err(|_| ReflowError::BorrowedLine)?;
    if Rc::ptr_eq(source, dest) {
        target.copy_cells_within(from, to, count, false);
    } else {
        let source = source.try_borrow().map_err(|_| ReflowError::BorrowedLine)?;
        target.copy_cells_from(&source, from, to, count, false);
    }
    Ok(())
}

fn compact(
    lines: &[LineRef],
    old: usize,
    new: usize,
    null: &CellData,
) -> Result<(usize, usize), ReflowError> {
    let mut dest_line = 0;
    let mut dest_col = get_wrapped_line_trimmed_length(lines, 0, old)?;
    let mut src_line = 1;
    let mut src_col = 0;
    while src_line < lines.len() {
        let length = get_wrapped_line_trimmed_length(lines, src_line, old)?;
        let remaining = length
            .checked_sub(src_col)
            .ok_or(ReflowError::InvalidRange)?;
        let count = remaining.min(new.checked_sub(dest_col).ok_or(ReflowError::InvalidRange)?);
        copy_cells(lines, src_line, src_col, dest_line, dest_col, count)?;
        dest_col += count;
        if dest_col == new {
            dest_line += 1;
            dest_col = 0;
        }
        src_col += count;
        if src_col == length {
            src_line += 1;
            src_col = 0;
        }
        if dest_col == 0
            && dest_line != 0
            && lines
                .get(dest_line - 1)
                .ok_or(ReflowError::InvalidRange)?
                .try_borrow()
                .map_err(|_| ReflowError::BorrowedLine)?
                .get_width(new - 1)
                == Some(2)
        {
            copy_cells(lines, dest_line - 1, new - 1, dest_line, 0, 1)?;
            dest_col = 1;
            lines[dest_line - 1]
                .try_borrow_mut()
                .map_err(|_| ReflowError::BorrowedLine)?
                .set_cell(new - 1, null);
        }
    }
    Ok((dest_line, dest_col))
}

/// Emit marker deletion events before applying the new order, as upstream does.
/// Listener dispatch errors propagate; events already emitted are not rolled back.
pub fn reflow_larger_create_new_layout(
    lines: &CircularList<LineRef>,
    to_remove: &[(usize, usize)],
) -> Result<NewLayoutResult, ReflowError> {
    let mut previous_end = 0;
    for &(start, count) in to_remove {
        if count == 0 || start < previous_end || start > lines.len() || count > lines.len() - start
        {
            return Err(ReflowError::InvalidRange);
        }
        previous_end = start + count;
    }
    let mut result = NewLayoutResult {
        layout: Vec::new(),
        count_removed: 0,
    };
    let mut next = 0;
    let mut i = 0;
    while i < lines.len() {
        if let Some(&(start, count)) = to_remove.get(next).filter(|&&(start, _)| start == i) {
            lines.on_delete_emitter.fire(&ListChange {
                index: start - result.count_removed,
                amount: count,
            })?;
            i += count;
            result.count_removed += count;
            next += 1;
        } else {
            result.layout.push(i);
            i += 1;
        }
    }
    Ok(result)
}

/// Snapshot slots before reordering, preserving object identity and empty slots.
/// A deletion observer can replace the buffer's list before this call, so layout
/// indices deliberately retain CircularList's cyclic, out-of-length reads.
pub fn reflow_larger_apply_new_layout(
    lines: &CircularList<LineRef>,
    new_layout: &[usize],
) -> Result<(), ReflowError> {
    let reordered: Vec<_> = new_layout.iter().map(|&i| lines.get(i)).collect();
    for (i, line) in reordered.into_iter().enumerate() {
        lines.set(i, line)?;
    }
    lines.set_length(new_layout.len());
    Ok(())
}

/// Compute shrink wrap points without splitting wide cells. Width one works for
/// narrow cells; wide cells are rejected instead of upstream's zero-progress loop.
pub fn reflow_smaller_get_new_line_lengths(
    wrapped_lines: &[LineRef],
    old_cols: usize,
    new_cols: usize,
) -> Result<Vec<usize>, ReflowError> {
    if old_cols == 0 || new_cols == 0 || new_cols > old_cols {
        return Err(ReflowError::InvalidColumns);
    }
    let lengths = (0..wrapped_lines.len())
        .map(|i| get_wrapped_line_trimmed_length(wrapped_lines, i, old_cols))
        .collect::<Result<Vec<_>, _>>()?;
    let cells_needed = lengths
        .iter()
        .try_fold(0usize, |sum, &n| sum.checked_add(n))
        .ok_or(ReflowError::InvalidRange)?;
    let mut result = Vec::new();
    let (mut src_col, mut src_line, mut available) = (0usize, 0, 0);
    while available < cells_needed {
        if cells_needed - available < new_cols {
            result.push(cells_needed - available);
            break;
        }
        src_col = src_col
            .checked_add(new_cols)
            .ok_or(ReflowError::InvalidRange)?;
        let old_length = *lengths.get(src_line).ok_or(ReflowError::InvalidRange)?;
        if src_col > old_length {
            src_col -= old_length;
            src_line += 1;
        }
        let ends_wide = wrapped_lines
            .get(src_line)
            .ok_or(ReflowError::InvalidRange)?
            .try_borrow()
            .map_err(|_| ReflowError::BorrowedLine)?
            .get_width(src_col.checked_sub(1).ok_or(ReflowError::InvalidRange)?)
            == Some(2);
        if ends_wide {
            if new_cols == 1 {
                return Err(ReflowError::InvalidColumns);
            }
            src_col -= 1;
        }
        let length = if ends_wide { new_cols - 1 } else { new_cols };
        result.push(length);
        available += length;
    }
    Ok(result)
}
