// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Alt-click cursor sequences. UTF-16 length, not cell count, drives alt-buffer
//! horizontal movement. The source's asymmetric backwards slices are retained.
use crate::buffer::{Buffer, BufferError, BufferService};
use std::{collections::TryReserveError, fmt};

#[derive(Debug)]
pub enum MoveToCellError {
    Buffer(BufferError),
    CoordinateRange,
    Allocation(TryReserveError),
}
impl fmt::Display for MoveToCellError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Buffer(e) => e.fmt(f),
            Self::CoordinateRange => {
                f.write_str("cursor movement coordinate or output length exceeds range")
            }
            Self::Allocation(e) => e.fmt(f),
        }
    }
}
impl std::error::Error for MoveToCellError {}
impl From<BufferError> for MoveToCellError {
    fn from(e: BufferError) -> Self {
        Self::Buffer(e)
    }
}
impl From<TryReserveError> for MoveToCellError {
    fn from(e: TryReserveError) -> Self {
        Self::Allocation(e)
    }
}
#[derive(Clone, Copy)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}
impl Direction {
    fn final_byte(self) -> char {
        match self {
            Self::Up => 'A',
            Self::Down => 'B',
            Self::Right => 'C',
            Self::Left => 'D',
        }
    }
}

/// Integer coordinates are zero-based and deliberately not clamped to the
/// viewport. The upstream normal-buffer tests include targetX==cols and
/// targetY==rows. Noninteger/nonfinite JS coordinates have no Rust equivalent.
/// Reverse-wraparound is not an input to the reference implementation.
pub fn move_to_cell_sequence(
    target_x: i64,
    target_y: i64,
    service: &BufferService,
    application_cursor: bool,
) -> Result<String, MoveToCellError> {
    let active = service.buffer();
    let buffer = active.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
    let start_x = i64::try_from(buffer.x).map_err(|_| MoveToCellError::CoordinateRange)?;
    let start_y = i64::try_from(buffer.y).map_err(|_| MoveToCellError::CoordinateRange)?;
    let cols = i64::try_from(service.cols()).map_err(|_| MoveToCellError::CoordinateRange)?;
    let rows = i64::try_from(service.rows()).map_err(|_| MoveToCellError::CoordinateRange)?;
    let mut result = String::new();
    if buffer.has_scrollback() {
        let direction = match start_y.cmp(&target_y) {
            std::cmp::Ordering::Equal if start_x > target_x => Direction::Left,
            std::cmp::Ordering::Greater => Direction::Left,
            _ => Direction::Right,
        };
        let count = if start_y == target_y {
            (i128::from(start_x) - i128::from(target_x)).abs()
        } else {
            let (end_x, begin_x) = if start_y > target_y {
                (target_x, start_x)
            } else {
                (start_x, target_x)
            };
            i128::from(cols) - i128::from(end_x)
                + (i128::from(start_y) - i128::from(target_y))
                    .abs()
                    .saturating_sub(1)
                    * i128::from(cols)
                + i128::from(begin_x)
        };
        append(&mut result, count, direction, application_cursor)?;
        return Ok(result);
    }
    let navigation = Navigation {
        buffer: &buffer,
        cols,
        rows,
    };
    let vertical = navigation.vertical_count(start_y, target_y)?;
    if vertical > 0 {
        let first = start_y - navigation.wrapped_rows(start_y)?;
        append(
            &mut result,
            navigation.line_length(start_x, start_y, start_x, first, false)?,
            Direction::Left,
            application_cursor,
        )?;
    }
    append(
        &mut result,
        vertical,
        if start_y > target_y {
            Direction::Up
        } else {
            Direction::Down
        },
        application_cursor,
    )?;
    let start_row = if vertical > 0 {
        target_y - navigation.wrapped_rows(target_y)?
    } else {
        start_y
    };
    let forward = (start_x < target_x && start_row <= target_y)
        || (start_x >= target_x && start_row < target_y);
    append(
        &mut result,
        navigation.line_length(start_x, start_row, target_x, target_y, forward)?,
        if forward {
            Direction::Right
        } else {
            Direction::Left
        },
        application_cursor,
    )?;
    Ok(result)
}
struct Navigation<'a> {
    buffer: &'a Buffer,
    cols: i64,
    rows: i64,
}
impl Navigation<'_> {
    fn wrapped(&self, row: i64) -> Result<bool, MoveToCellError> {
        self.buffer.lines.get_signed(row).map_or(Ok(false), |line| {
            Ok(line
                .try_borrow()
                .map_err(|_| BufferError::BorrowedLine)?
                .is_wrapped)
        })
    }
    fn wrapped_rows(&self, mut row: i64) -> Result<i64, MoveToCellError> {
        let mut count = 0;
        while self.wrapped(row)? && row >= 0 && row < self.rows {
            count += 1;
            row -= 1;
        }
        Ok(count)
    }
    fn vertical_count(&self, start_y: i64, target_y: i64) -> Result<i128, MoveToCellError> {
        let start = start_y - self.wrapped_rows(start_y)?;
        let end = target_y - self.wrapped_rows(target_y)?;
        let distance = (i128::from(start) - i128::from(end)).abs();
        let direction = if start_y > target_y { -1 } else { 1 };
        let mut wrapped = 0;
        for offset in 0..distance {
            let row = i128::from(start) + direction * offset;
            if self.wrapped(i64::try_from(row).map_err(|_| MoveToCellError::CoordinateRange)?)? {
                wrapped += 1;
            }
        }
        Ok(distance - wrapped)
    }
    fn translated_length(&self, row: i64, start: i64, end: i64) -> Result<i128, MoveToCellError> {
        let Some(line) = self.buffer.lines.get_signed(row) else {
            return Ok(0);
        };
        if start >= end {
            return Ok(0);
        }
        let prefix = if start < 0 {
            i128::from(end.min(0)) - i128::from(start)
        } else {
            0
        };
        let start = usize::try_from(start.max(0)).map_err(|_| MoveToCellError::CoordinateRange)?;
        let end = usize::try_from(end.max(0)).map_err(|_| MoveToCellError::CoordinateRange)?;
        let line = line.try_borrow().map_err(|_| BufferError::BorrowedLine)?;
        Ok(prefix
            + line
                .translate_to_string(false, Some(start), Some(end), None)
                .len() as i128)
    }
    fn line_length(
        &self,
        mut start_col: i64,
        mut row: i64,
        end_col: i64,
        end_row: i64,
        forward: bool,
    ) -> Result<i128, MoveToCellError> {
        let mut col = start_col;
        let mut length = 0;
        while (col != end_col || row != end_row)
            && row >= 0
            && (row as u128) < self.buffer.lines.len() as u128
        {
            col += if forward { 1 } else { -1 };
            if forward && col > self.cols - 1 {
                length += self.translated_length(row, start_col, col)?;
                col = 0;
                start_col = 0;
                row += 1;
            } else if !forward && col < 0 {
                length += self.translated_length(row, 0, start_col + 1)?;
                col = self.cols - 1;
                start_col = col;
                row -= 1;
            }
        }
        Ok(length + self.translated_length(row, start_col, col)?)
    }
}
fn append(
    output: &mut String,
    count: i128,
    direction: Direction,
    application: bool,
) -> Result<(), MoveToCellError> {
    if count <= 0 {
        return Ok(());
    }
    let count = usize::try_from(count).map_err(|_| MoveToCellError::CoordinateRange)?;
    let bytes = count
        .checked_mul(3)
        .ok_or(MoveToCellError::CoordinateRange)?;
    output.try_reserve(bytes)?;
    for _ in 0..count {
        output.push('\x1b');
        output.push(if application { 'O' } else { '[' });
        output.push(direction.final_byte());
    }
    Ok(())
}
