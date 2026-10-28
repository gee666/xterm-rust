// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! UTF-16 string indexes and cell columns advance separately for wide/combined cells.
use super::{model::Position, service::SelectionError};
use xterm_core::buffer::{Buffer, BufferLine};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WordPosition {
    /// Can be negative when the word starts on a preceding wrapped row.
    pub start: i64,
    pub length: u32,
}

pub fn get_word_at(
    buffer: &Buffer,
    position: Position,
    columns: u32,
    separators: &[u16],
    allow_whitespace: bool,
) -> Result<Option<WordPosition>, SelectionError> {
    if position.column >= i64::from(columns) || columns == 0 {
        return Ok(None);
    }
    let Some(mut word) = local_word(buffer, position, columns, separators, allow_whitespace)?
    else {
        return Ok(None);
    };
    let mut above = position.row;
    let mut above_word = word;
    // Iteration preserves recursive upstream results without allowing a long
    // wrapped line from terminal output to exhaust the Rust call stack.
    while above_word.start == 0 && code(buffer, above, 0)? != Some(32) {
        if !is_wrapped(buffer, above)?
            || above <= 0
            || code(buffer, above - 1, columns - 1)? == Some(32)
        {
            break;
        }
        let Some(previous) = local_word(
            buffer,
            Position::new(i64::from(columns - 1), above - 1),
            columns,
            separators,
            false,
        )?
        else {
            break;
        };
        let offset = i64::from(columns)
            .checked_sub(previous.start)
            .ok_or(SelectionError::Arithmetic)?;
        word.start = word
            .start
            .checked_sub(offset)
            .ok_or(SelectionError::Arithmetic)?;
        word.length = add_length(word.length, offset)?;
        above -= 1;
        above_word = previous;
    }
    let mut below = position.row;
    let mut below_word = WordPosition {
        start: word.start,
        length: word.length,
    };
    while below_word.start.checked_add(i64::from(below_word.length)) == Some(i64::from(columns))
        && code(buffer, below, columns - 1)? != Some(32)
    {
        let next_row = below.checked_add(1).ok_or(SelectionError::Arithmetic)?;
        if !is_wrapped(buffer, next_row)? || code(buffer, next_row, 0)? == Some(32) {
            break;
        }
        let Some(next) = local_word(
            buffer,
            Position::new(0, next_row),
            columns,
            separators,
            false,
        )?
        else {
            break;
        };
        word.length = word
            .length
            .checked_add(next.length)
            .ok_or(SelectionError::Arithmetic)?;
        below = next_row;
        below_word = next;
    }
    Ok(Some(word))
}

fn add_length(length: u32, amount: i64) -> Result<u32, SelectionError> {
    length
        .checked_add(u32::try_from(amount).map_err(|_| SelectionError::Arithmetic)?)
        .ok_or(SelectionError::Arithmetic)
}
fn line_at(buffer: &Buffer, row: i64) -> Option<xterm_core::buffer::screen::LineRef> {
    usize::try_from(row)
        .ok()
        .and_then(|row| buffer.lines.get(row))
}
fn code(buffer: &Buffer, row: i64, column: u32) -> Result<Option<u32>, SelectionError> {
    let Some(line) = line_at(buffer, row) else {
        return Ok(None);
    };
    let line = line.try_borrow().map_err(|_| SelectionError::Borrowed)?;
    Ok(line.get_code_point(usize::try_from(column).map_err(|_| SelectionError::Arithmetic)?))
}
fn is_wrapped(buffer: &Buffer, row: i64) -> Result<bool, SelectionError> {
    let Some(line) = line_at(buffer, row) else {
        return Ok(false);
    };
    let wrapped = line
        .try_borrow()
        .map_err(|_| SelectionError::Borrowed)?
        .is_wrapped;
    Ok(wrapped)
}
struct WordCell {
    width: i32,
    chars: Vec<u16>,
}
impl WordCell {
    fn get_width(&self) -> i32 {
        self.width
    }
    fn get_chars(&self) -> &[u16] {
        &self.chars
    }
}
fn cell(line: &BufferLine, column: i64) -> WordCell {
    let column = usize::try_from(column).ok();
    // Word lookup needs no attributes. Reading just these fields avoids cloning
    // extended attributes and copying a combined string a second time.
    WordCell {
        width: column
            .and_then(|column| line.get_width(column))
            .unwrap_or(0),
        chars: column
            .and_then(|column| line.get_string(column))
            .unwrap_or_default(),
    }
}
fn separator(cell: &WordCell, separators: &[u16]) -> bool {
    if cell.get_width() == 0 {
        return false;
    }
    let chars = cell.get_chars();
    chars.is_empty()
        || separators
            .windows(chars.len())
            .any(|candidate| candidate == chars)
}
fn character_index(line: &BufferLine, column: i64) -> Result<i64, SelectionError> {
    let mut index = column;
    for current in 0..=column {
        let cell = cell(line, current);
        let length =
            i64::try_from(cell.get_chars().len()).map_err(|_| SelectionError::Arithmetic)?;
        if cell.get_width() == 0 {
            index -= 1;
        } else if length > 1 && column != current {
            index += length - 1;
        }
    }
    Ok(index)
}
fn unit(text: &[u16], index: i64) -> Option<u16> {
    usize::try_from(index)
        .ok()
        .and_then(|index| text.get(index))
        .copied()
}

fn local_word(
    buffer: &Buffer,
    position: Position,
    columns: u32,
    separators: &[u16],
    allow_whitespace: bool,
) -> Result<Option<WordPosition>, SelectionError> {
    let Some(line) = line_at(buffer, position.row) else {
        return Ok(None);
    };
    let line = line.try_borrow().map_err(|_| SelectionError::Borrowed)?;
    let text = line.translate_to_string(false, None, None, None);
    let mut start_index = character_index(&line, position.column)?;
    let mut end_index = start_index;
    let offset = position.column - start_index;
    let (mut left_wide, mut right_wide, mut left_long, mut right_long) = (0, 0, 0, 0);
    let text_length = i64::try_from(text.len()).map_err(|_| SelectionError::Arithmetic)?;
    if unit(&text, start_index) == Some(32) {
        while start_index > 0 && unit(&text, start_index - 1) == Some(32) {
            start_index -= 1;
        }
        while end_index < text_length && unit(&text, end_index + 1) == Some(32) {
            end_index += 1;
        }
    } else {
        let (mut start_column, mut end_column) = (position.column, position.column);
        if cell(&line, start_column).get_width() == 0 {
            left_wide += 1;
            start_column -= 1;
        }
        if cell(&line, end_column).get_width() == 2 {
            right_wide += 1;
            end_column += 1;
        }
        let length = i64::try_from(cell(&line, end_column).get_chars().len())
            .map_err(|_| SelectionError::Arithmetic)?;
        if length > 1 {
            right_long += length - 1;
            end_index += length - 1;
        }
        while start_column > 0
            && start_index > 0
            && !separator(&cell(&line, start_column - 1), separators)
        {
            let cell = cell(&line, start_column - 1);
            let length =
                i64::try_from(cell.get_chars().len()).map_err(|_| SelectionError::Arithmetic)?;
            if cell.get_width() == 0 {
                left_wide += 1;
                start_column -= 1;
            } else if length > 1 {
                left_long += length - 1;
                start_index -= length - 1;
            }
            start_index -= 1;
            start_column -= 1;
        }
        let line_length = i64::try_from(line.len()).map_err(|_| SelectionError::Arithmetic)?;
        while end_column < line_length
            && end_index + 1 < text_length
            && !separator(&cell(&line, end_column + 1), separators)
        {
            let cell = cell(&line, end_column + 1);
            let length =
                i64::try_from(cell.get_chars().len()).map_err(|_| SelectionError::Arithmetic)?;
            if cell.get_width() == 2 {
                right_wide += 1;
                end_column += 1;
            } else if length > 1 {
                right_long += length - 1;
                end_index += length - 1;
            }
            end_index += 1;
            end_column += 1;
        }
    }
    end_index += 1;
    let start = start_index + offset - left_wide + left_long;
    let length = i64::from(columns)
        .min(end_index - start_index + left_wide + right_wide - left_long - right_long);
    if !allow_whitespace {
        let begin = usize::try_from(start_index.max(0))
            .map_err(|_| SelectionError::Arithmetic)?
            .min(text.len());
        let end = usize::try_from(end_index.max(0))
            .map_err(|_| SelectionError::Arithmetic)?
            .min(text.len());
        if begin >= end || text[begin..end].iter().all(|&unit| js_whitespace(unit)) {
            return Ok(None);
        }
    }
    Ok(Some(WordPosition {
        start,
        length: u32::try_from(length).map_err(|_| SelectionError::Arithmetic)?,
    }))
}
fn js_whitespace(unit: u16) -> bool {
    matches!(unit, 0x0009..=0x000d | 0x0020 | 0x00a0 | 0x1680 | 0x2000..=0x200a | 0x2028 | 0x2029 | 0x202f | 0x205f | 0x3000 | 0xfeff)
}
