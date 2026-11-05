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
    style_position: (usize, usize),
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
    let mut h = Handler::new(buffer, rows, range.start.y);
    for row in range.start.y..=range.end.y {
        if let Some(len) = buffer.capture_line(row)? {
            let start = if row == range.start.y {
                range.start.x
            } else {
                0
            };
            let end = if row == range.end.y { range.end.x } else { len };
            for col in start..end {
                if let Some(cell) = buffer.captured_cell(row, col)? {
                    h.next(&cell, row, col);
                } else {
                    buffer.missing_cell(row, col)?;
                }
            }
        }
        h.row_end(row, row == range.end.y)?;
    }
    h.finish(attrs, exclude_cursor)
}

/// Serialize an equal finite numeric row range with no final cursor movement.
/// Raw row lookup preserves browser ring-buffer aliases before index conversion.
/// Values outside the JS safe domain are rejected rather than reproducing a
/// nonprogressing `row++` loop. This does not support arbitrary numeric ranges.
pub fn serialize_numeric_row(
    buffer: &dyn BufferAccess,
    cols: usize,
    rows: usize,
    attrs: &AttributeData,
    row: f64,
) -> Result<Vec<u16>, SerializeError> {
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;
    if !row.is_finite() || row.abs() > MAX_SAFE_INTEGER {
        return Err(SerializeError::InvalidRange);
    }
    // No final cursor movement or wrapping lookahead needs this row's origin.
    // Lookup and diagnostics keep the raw key, as does the floating-point
    // trimming expression, whose rounding can expose an absent array entry.
    let mut h = Handler::new(buffer, rows, 0);
    if buffer.capture_numeric_line(row)?.is_some() {
        for col in 0..cols {
            if let Some(cell) = buffer.captured_numeric_cell(row, col)? {
                h.next(&cell, 0, col);
            } else {
                buffer.missing_numeric_cell(row, col)?;
            }
        }
    }
    h.row_end(0, true)?;
    h.finish_numeric_row(attrs, row)
}

impl<'a> Handler<'a> {
    fn new(buffer: &'a dyn BufferAccess, rows: usize, first: usize) -> Self {
        Self {
            buffer,
            rows,
            first,
            current: Vec::new(),
            all: Vec::new(),
            nulls: 0,
            cursor_style: CellData::default(),
            style_position: (0, 0),
            background: CellData::default(),
            cursor: (first, 0),
            content_cursor: (first, 0),
        }
    }

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
            self.style_position = (row, col);
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
            if row - self.first >= self.buffer.current_rows(self.rows)? {
                if let Some(cell) = self
                    .buffer
                    .cell(self.style_position.0, self.style_position.1)?
                {
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

    fn finish(
        mut self,
        attrs: &AttributeData,
        exclude_cursor: bool,
    ) -> Result<Vec<u16>, SerializeError> {
        let row_end = if self.buffer.current_length()?.saturating_sub(self.first)
            <= self.buffer.current_rows(self.rows)?
        {
            self.cursor = self.content_cursor;
            self.content_cursor.0 + 1 - self.first
        } else {
            self.all.len()
        };
        self.finish_output(attrs, exclude_cursor, row_end, false)
    }

    fn finish_numeric_row(
        self,
        attrs: &AttributeData,
        row: f64,
    ) -> Result<Vec<u16>, SerializeError> {
        let trim = self.buffer.current_length()? as f64 - row
            <= self.buffer.current_rows(self.rows)? as f64;
        // In source this is a floating loop limit. Rounding above1 performs a
        // second iteration, appending the missing array entry as `undefined`.
        let undefined_row = trim && row + 1.0 - row > 1.0;
        self.finish_output(
            attrs,
            true,
            if undefined_row { 2 } else { 1 },
            undefined_row,
        )
    }

    fn finish_output(
        self,
        attrs: &AttributeData,
        exclude_cursor: bool,
        row_end: usize,
        undefined_row: bool,
    ) -> Result<Vec<u16>, SerializeError> {
        let mut output = Vec::new();
        for (i, (text, separator)) in self.all.iter().take(row_end).enumerate() {
            output.extend(text);
            if i + 1 < row_end {
                append(&mut output, separator);
            }
        }
        if undefined_row {
            append(&mut output, "undefined");
        }
        if !exclude_cursor {
            let cursor = self.buffer.current_cursor()?;
            movement(&mut output, cursor.y, self.cursor.0, 'B', 'A');
            movement(&mut output, cursor.x, self.cursor.1, 'C', 'D');
        }
        let attrs = self.buffer.current_attrs(attrs)?;
        let sgr = style::diff(&attrs, &self.cursor_style);
        if !sgr.is_empty() {
            append(&mut output, &format!("\x1b[{sgr}m"));
        }
        Ok(output)
    }
}

fn movement(out: &mut Vec<u16>, target: usize, current: usize, positive: char, negative: char) {
    match target.cmp(&current) {
        std::cmp::Ordering::Greater => append(out, &format!("\x1b[{}{positive}", target - current)),
        std::cmp::Ordering::Less => append(out, &format!("\x1b[{}{negative}", current - target)),
        std::cmp::Ordering::Equal => {}
    }
}
