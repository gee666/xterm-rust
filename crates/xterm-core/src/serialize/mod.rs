// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Serialization reads live Rust buffer cells. Output is UTF-16 to preserve JS
//! strings, including lone surrogates; callers choose their encoding at the edge.
mod addon;
mod ansi;
pub use addon::{serialize_live_html, serialize_live_state, AddonSerializeError, SerializeAddon};
pub use ansi::serialize_numeric_row;
mod buffer;
mod html;
mod html_rows;
pub use html_rows::HtmlNumericRange;
mod live_modes;
pub use live_modes::serialize_live_modes;
mod view;
pub use html::{serialize_html, serialize_html_numeric_rows};
pub use view::PublicBufferView;
mod style;
mod terminal;
pub use terminal::{serialize_terminal, MouseTracking, SerializeModes, SerializeState};

use crate::buffer::{AttributeData, CellData};

/// Read-only buffer operations used by both serializers. Browser callbacks may
/// mutate the terminal during a diagnostic; late state reads are fallible hooks.
pub trait BufferAccess {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError>;
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError>;
    /// Capture one traversal row. Source keeps this line through host warnings;
    /// lookahead and style reads continue to use the ordinary live methods.
    fn capture_line(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        self.line_len(row)
    }
    /// Read from the row most recently captured for this traversal.
    fn captured_cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        self.cell(row, col)
    }
    /// Numeric browser row keys must reach the host's array/ring arithmetic
    /// before conversion. Ordinary integral-only adapters use missing rows for
    /// keys they cannot represent; source-backed ring adapters override this.
    fn capture_numeric_line(&self, row: f64) -> Result<Option<usize>, SerializeError> {
        numeric_index(row).map_or(Ok(None), |row| self.capture_line(row))
    }
    fn captured_numeric_cell(
        &self,
        row: f64,
        col: usize,
    ) -> Result<Option<CellData>, SerializeError> {
        numeric_index(row).map_or(Ok(None), |row| self.captured_cell(row, col))
    }
    /// A browser warning must interpolate the original coordinate, not its
    /// wrapped storage index. Native hosts keep their ordinary diagnostic hook.
    fn missing_numeric_cell(&self, row: f64, col: usize) -> Result<(), SerializeError> {
        numeric_index(row).map_or(Ok(()), |row| self.missing_cell(row, col))
    }
    fn is_wrapped(&self, row: usize) -> Result<bool, SerializeError>;
    fn cursor_x(&self) -> usize;
    fn cursor_y(&self) -> usize;
    fn base_y(&self) -> usize;
    /// Native callers supply immutable dimensions; browser hosts read live rows.
    fn current_rows(&self, fallback: usize) -> Result<usize, SerializeError> {
        Ok(fallback)
    }
    fn current_length(&self) -> Result<usize, SerializeError> {
        Ok(self.len())
    }
    fn current_cursor(&self) -> Result<Position, SerializeError> {
        Ok(Position {
            x: self.cursor_x(),
            y: self.base_y() + self.cursor_y(),
        })
    }
    /// Cursor attributes are restored after traversal, not captured before it.
    fn current_attrs(&self, fallback: &AttributeData) -> Result<AttributeData, SerializeError> {
        Ok(fallback.clone())
    }
    /// Browser hosts preserve the source diagnostic for a missing traversed cell.
    fn missing_cell(&self, _row: usize, _col: usize) -> Result<(), SerializeError> {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// Final column is exclusive, as in xterm's IBufferRange.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CellRange {
    pub start: Position,
    pub end: Position,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LineRange {
    pub start: usize,
    pub end: usize,
}

#[derive(Clone, Debug, Default)]
pub struct SerializeOptions {
    pub range: Option<LineRange>,
    pub scrollback: Option<isize>,
    pub exclude_modes: bool,
    pub exclude_alt_buffer: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HtmlRange {
    pub start_line: usize,
    pub end_line: usize,
    pub start_col: usize,
}

#[derive(Clone, Debug, Default)]
pub struct HtmlOptions {
    pub scrollback: Option<isize>,
    pub only_selection: bool,
    pub include_global_background: bool,
    pub range: Option<HtmlRange>,
}

/// Numeric configuration uses JS formatting; a host can also supply an exact
/// already-coerced value because source option concatenation accepts raw values.
#[derive(Clone, Debug)]
pub enum HtmlFontSize {
    Number(f64),
    Text(Vec<u16>),
}
impl From<f64> for HtmlFontSize {
    fn from(value: f64) -> Self {
        Self::Number(value)
    }
}

/// String and palette formatting belongs to the host theme, not core policy.
#[derive(Clone, Debug)]
pub struct HtmlConfig {
    pub foreground: Option<Vec<u16>>,
    pub background: Option<Vec<u16>>,
    pub font_family: Vec<u16>,
    pub font_size: HtmlFontSize,
    /// None selects the headless default palette; Some is a full theme snapshot.
    pub ansi_colors: Option<[String; 256]>,
}

#[derive(Clone, Debug)]
pub enum SerializeError {
    InvalidRange,
    BorrowedLine,
    MissingWrappedLine,
    BufferView(std::rc::Rc<crate::buffer::BufferError>),
}
impl PartialEq for SerializeError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::InvalidRange, Self::InvalidRange)
            | (Self::BorrowedLine, Self::BorrowedLine)
            | (Self::MissingWrappedLine, Self::MissingWrappedLine) => true,
            // Transported callback errors retain identity, rather than matching
            // unrelated failures just because their messages happen to agree.
            (Self::BufferView(a), Self::BufferView(b)) => std::rc::Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}
impl Eq for SerializeError {}
impl From<crate::buffer::BufferError> for SerializeError {
    fn from(error: crate::buffer::BufferError) -> Self {
        Self::BufferView(std::rc::Rc::new(error))
    }
}
impl std::fmt::Display for SerializeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InvalidRange => "Invalid array length",
            Self::BorrowedLine => "buffer line already borrowed",
            Self::MissingWrappedLine => "Cannot read properties of undefined (reading 'isWrapped')",
            Self::BufferView(error) => return error.fmt(f),
        })
    }
}
impl std::error::Error for SerializeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::BufferView(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}

pub fn serialize_buffer(
    buffer: &dyn BufferAccess,
    cols: usize,
    rows: usize,
    attrs: &AttributeData,
    range: CellRange,
    exclude_final_cursor: bool,
) -> Result<Vec<u16>, SerializeError> {
    if range.start.y > range.end.y {
        return Err(SerializeError::InvalidRange);
    }
    ansi::serialize(buffer, cols, rows, attrs, range, exclude_final_cursor)
}

fn numeric_index(row: f64) -> Option<usize> {
    if row.is_finite() && row >= 0.0 && row.fract() == 0.0 && row < usize::MAX as f64 {
        Some(row as usize)
    } else {
        None
    }
}

pub(crate) fn append(output: &mut Vec<u16>, text: &str) {
    output.extend(text.encode_utf16());
}

pub(crate) fn scrollback_start(length: usize, rows: usize, scrollback: Option<isize>) -> usize {
    let retained = scrollback.map_or(length, |count| {
        if count >= 0 {
            rows.saturating_add(count as usize).min(length)
        } else {
            rows.saturating_sub(count.unsigned_abs()).min(length)
        }
    });
    length - retained
}
