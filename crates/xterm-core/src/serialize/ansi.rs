// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use super::{append, style, BufferAccess, CellRange, SerializeError};
use crate::buffer::{AttributeData, CellData};

struct Handler<'a> {
    buffer: &'a dyn BufferAccess,
    rows: usize,
    first: usize,
    current: Vec<u16>,
    all: Vec<(Vec<u16>, String)>,
    nulls: usize,
    cursor_style: CellData,
    style_position: Option<(usize, usize)>,
    background: CellData,
    cursor: (usize, usize),
    content_cursor: (usize, usize),
}

pub(super) fn serialize(
    buffer: &dyn BufferAccess,
    _cols: usize,
    rows: usize,
    attrs: &AttributeData,
    range: CellRange,
    exclude_cursor: bool,
) -> Result<Vec<u16>, SerializeError> {
    let mut h = Handler {
        buffer,
        rows,
        first: range.start.y,
        current: Vec::new(),
        all: Vec::new(),
        nulls: 0,
        cursor_style: CellData::default(),
        style_position: None,
        background: CellData::default(),
        cursor: (range.start.y, 0),
        content_cursor: (range.start.y, 0),
    };
    for row in range.start.y..=range.end.y {
        if let Some(len) = buffer.line_len(row)? {
            let start = if row == range.start.y {
                range.start.x
            } else {
                0
            };
            let end = if row == range.end.y { range.end.x } else { len };
            for col in start..end {
                if let Some(cell) = buffer.cell(row, col)? {
                    h.next(&cell, row, col);
                }
            }
        }
        h.row_end(row, row == range.end.y)?;
    }
    Ok(h.finish(attrs, exclude_cursor))
}

impl Handler<'_> {
    fn emit(&mut self, s: &str) {
        append(&mut self.current, s);
    }

    fn next(&mut self, cell: &CellData, row: usize, col: usize) {
        let width = cell.get_width();
        if width == 0 {
            return;
        }
        let chars = cell.get_chars();
        let empty = chars.is_empty();
        let sgr = style::diff(cell, &self.cursor_style);
        let changed = if empty {
            !style::equal_bg(&self.cursor_style, cell)
        } else {
            !sgr.is_empty()
        };
        if changed {
            if self.nulls > 0 {
                if !style::equal_bg(&self.cursor_style, &self.background) {
                    self.emit(&format!("\x1b[{}X", self.nulls));
                }
                self.emit(&format!("\x1b[{}C", self.nulls));
                self.nulls = 0;
            }
            self.content_cursor = (row, col);
            self.cursor = (row, col);
            self.emit(&format!("\x1b[{sgr}m"));
            self.cursor_style = cell.clone();
            self.style_position = Some((row, col));
        }
        if empty {
            self.nulls = self.nulls.saturating_add(width.max(0) as usize);
        } else {
            if self.nulls > 0 {
                if !style::equal_bg(&self.cursor_style, &self.background) {
                    self.emit(&format!("\x1b[{}X", self.nulls));
                }
                self.emit(&format!("\x1b[{}C", self.nulls));
                self.nulls = 0;
            }
            self.current.extend(chars);
            self.content_cursor = (row, col.saturating_add(width.max(0) as usize));
            self.cursor = self.content_cursor;
        }
    }

    fn row_end(&mut self, row: usize, last: bool) -> Result<(), SerializeError> {
        if self.nulls > 0 && !style::equal_bg(&self.cursor_style, &self.background) {
            self.emit(&format!("\x1b[{}X", self.nulls));
        }
        let mut separator = String::new();
        if !last {
            if row - self.first >= self.rows {
                let cell = match self.style_position {
                    Some((row, col)) => self.buffer.cell(row, col)?,
                    None => self.buffer.reference_cell(0)?,
                };
                if let Some(cell) = cell {
                    self.background = cell;
                }
            }
            // Source reads nextLine.isWrapped even when a caller's range runs
            // past the buffer. Preserve that failure instead of omitting rows.
            if self.buffer.line_len(row + 1)?.is_none() {
                return Err(SerializeError::MissingWrappedLine);
            }
            if !self.buffer.is_wrapped(row + 1)? {
                separator.push_str("\r\n");
                self.cursor = (row + 1, 0);
            } else {
                separator = self.wrap_separator(row)?;
            }
        }
        self.all
            .push((std::mem::take(&mut self.current), separator));
        self.nulls = 0;
        Ok(())
    }

    fn wrap_separator(&mut self, row: usize) -> Result<String, SerializeError> {
        let len = self.buffer.line_len(row)?.unwrap_or(0);
        let last = match len.checked_sub(1) {
            Some(c) => self.buffer.cell(row, c)?,
            None => None,
        };
        let second = match len.checked_sub(2) {
            Some(c) => self.buffer.cell(row, c)?,
            None => None,
        };
        let next = self.buffer.cell(row + 1, 0)?;
        let mut valid = false;
        if let (Some(last), Some(next)) = (last.as_ref(), next.as_ref()) {
            let wide = next.get_width() > 1;
            // Preserve the source ternary's precedence, including its empty-cell arm.
            let enough = if !next.get_chars().is_empty() && wide {
                self.nulls <= 1
            } else {
                self.nulls == 0
            };
            if enough {
                let content = |c: &CellData| !c.get_chars().is_empty() || c.get_width() == 0;
                valid = content(last) && style::equal_bg(last, next);
                if let Some(second) = second.as_ref() {
                    valid |= wide
                        && content(second)
                        && style::equal_bg(last, next)
                        && style::equal_bg(second, next);
                }
            }
        }
        if valid {
            return Ok(String::new());
        }
        let mut out = "-".repeat(self.nulls + 1);
        out.push_str("\x1b[1D\x1b[1X");
        if self.nulls > 0 {
            let offset = len.saturating_sub(self.nulls);
            out.push_str(&format!(
                "\x1b[A\x1b[{offset}C\x1b[{}X\x1b[{offset}D\x1b[B",
                self.nulls
            ));
        }
        self.content_cursor = (row + 1, 0);
        self.cursor = (row + 1, 0);
        Ok(out)
    }

    fn finish(mut self, attrs: &AttributeData, exclude_cursor: bool) -> Vec<u16> {
        let row_end = if self.buffer.len().saturating_sub(self.first) <= self.rows {
            self.cursor = self.content_cursor;
            self.content_cursor.0 + 1 - self.first
        } else {
            self.all.len()
        };
        let mut output = Vec::new();
        for (i, (text, separator)) in self.all.iter().take(row_end).enumerate() {
            output.extend(text);
            if i + 1 < row_end {
                append(&mut output, separator);
            }
        }
        if !exclude_cursor {
            movement(
                &mut output,
                self.buffer.base_y() + self.buffer.cursor_y(),
                self.cursor.0,
                'B',
                'A',
            );
            movement(&mut output, self.buffer.cursor_x(), self.cursor.1, 'C', 'D');
        }
        let sgr = style::diff(attrs, &self.cursor_style);
        if !sgr.is_empty() {
            append(&mut output, &format!("\x1b[{sgr}m"));
        }
        output
    }
}

fn movement(out: &mut Vec<u16>, target: usize, current: usize, positive: char, negative: char) {
    match target.cmp(&current) {
        std::cmp::Ordering::Greater => append(out, &format!("\x1b[{}{positive}", target - current)),
        std::cmp::Ordering::Less => append(out, &format!("\x1b[{}{negative}", current - target)),
        std::cmp::Ordering::Equal => {}
    }
}
