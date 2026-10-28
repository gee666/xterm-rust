// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Serialization reads live Rust buffer cells. Output is UTF-16 to preserve JS
//! strings, including lone surrogates; callers choose their encoding at the edge.
mod addon;
mod ansi;
pub use addon::{serialize_live_html, serialize_live_state, AddonSerializeError, SerializeAddon};
mod buffer;
mod html;
mod view;
pub use html::serialize_html;
mod style;
mod terminal;
pub use terminal::{serialize_terminal, MouseTracking, SerializeModes, SerializeState};

use crate::buffer::{AttributeData, CellData};

/// Read-only buffer operations used by both serializers. Implementations should
/// return a consistent view for the duration of a serialization call.
pub trait BufferAccess {
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError>;
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError>;
    fn is_wrapped(&self, row: usize) -> Result<bool, SerializeError>;
    fn cursor_x(&self) -> usize;
    fn cursor_y(&self) -> usize;
    fn base_y(&self) -> usize;
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
