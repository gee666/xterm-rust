// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Terminal buffer state. Row references preserve JavaScript line identity.
use super::{AttributeData, BufferLine, CellData};
use crate::{
    charset::Charset,
    circular_list::{CircularList, CircularListError},
    event::{EmitError, Emitter},
    options::{OptionError, OptionsService},
};
use std::{
    cell::{Cell, RefCell},
    collections::BTreeSet,
    fmt,
    rc::Rc,
};

#[path = "screen_markers.rs"]
mod markers;
#[path = "screen_navigation.rs"]
mod navigation;
pub use navigation::WrappedRange;
#[path = "screen_options.rs"]
mod options;
#[path = "screen_reflow.rs"]
mod reflow;
#[path = "screen_resize.rs"]
mod resize;
#[path = "screen_shared.rs"]
mod shared;

pub type LineRef = Rc<RefCell<BufferLine>>;
pub const MAX_BUFFER_SIZE: usize = u32::MAX as usize;

/// Live dimensions owned by the host service, distinct from a Buffer's last resize.
#[derive(Debug)]
pub struct BufferDimensions {
    pub cols: Cell<usize>,
    pub rows: Cell<usize>,
}
impl BufferDimensions {
    pub fn new(cols: usize, rows: usize) -> Self {
        Self {
            cols: Cell::new(cols),
            rows: Cell::new(rows),
        }
    }
}

#[derive(Clone, Debug)]
pub enum BufferError {
    InvalidDimensions,
    InvalidOption(&'static str),
    MissingLine(usize),
    BorrowedLine,
    CleanupAlreadyAttached,
    CleanupQueueBusy,
    SizeOverflow,
    Options(OptionError),
    List(CircularListError),
    Dispatch(EmitError),
    Reflow(super::reflow::ReflowError),
}
impl fmt::Display for BufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => f.write_str("buffer dimensions must be positive"),
            Self::InvalidOption(key) => write!(f, "invalid buffer option {key}"),
            Self::MissingLine(index) => write!(f, "missing buffer line {index}"),
            Self::BorrowedLine => f.write_str("buffer line already borrowed"),
            Self::CleanupAlreadyAttached => {
                f.write_str("buffer already has a host cleanup scheduler")
            }
            Self::CleanupQueueBusy => {
                f.write_str("buffer cleanup requires an empty dedicated task queue")
            }
            Self::SizeOverflow => f.write_str("buffer size arithmetic overflow"),
            Self::Options(error) => error.fmt(f),
            Self::List(error) => error.fmt(f),
            Self::Dispatch(error) => error.fmt(f),
            Self::Reflow(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for BufferError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Options(error) => Some(error),
            Self::List(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Reflow(error) => Some(error),
            _ => None,
        }
    }
}
impl From<OptionError> for BufferError {
    fn from(e: OptionError) -> Self {
        Self::Options(e)
    }
}
impl From<CircularListError> for BufferError {
    fn from(e: CircularListError) -> Self {
        Self::List(e)
    }
}
impl From<EmitError> for BufferError {
    fn from(e: EmitError) -> Self {
        Self::Dispatch(e)
    }
}
impl From<super::reflow::ReflowError> for BufferError {
    fn from(e: super::reflow::ReflowError) -> Self {
        Self::Reflow(e)
    }
}

pub struct Buffer {
    pub lines: CircularList<LineRef>,
    pub ydisp: usize,
    pub ybase: usize,
    pub y: usize,
    pub x: usize,
    pub scroll_bottom: usize,
    pub scroll_top: usize,
    pub tabs: BTreeSet<usize>,
    // Source reflow can decrement this below zero for an empty wrapped row.
    pub saved_y: i64,
    pub saved_x: usize,
    pub saved_cur_attr_data: AttributeData,
    pub saved_charset: Option<Charset>,
    pub saved_charsets: Vec<Option<Charset>>,
    pub saved_glevel: usize,
    pub saved_origin_mode: bool,
    pub saved_wraparound_mode: bool,
    pub markers: Rc<RefCell<Vec<super::marker::Marker>>>,
    pub host_dimensions: Rc<BufferDimensions>,
    pub options: Rc<OptionsService>,
    cols: usize,
    rows: usize,
    has_scrollback: bool,
    is_clearing: Rc<Cell<bool>>,
    cleanup_position: usize,
    cleanup_pending: bool,
    disposed: bool,
    pub(crate) cleanup_events: Rc<Emitter<bool>>,
    pub(crate) cleanup_attached: Rc<Cell<bool>>,
}
impl Buffer {
    pub fn new(
        has_scrollback: bool,
        cols: usize,
        rows: usize,
        options: Rc<OptionsService>,
    ) -> Result<Self, BufferError> {
        if cols == 0 || rows == 0 {
            return Err(BufferError::InvalidDimensions);
        }
        let mut buffer = Self {
            lines: CircularList::new(rows)?,
            ydisp: 0,
            ybase: 0,
            y: 0,
            x: 0,
            scroll_bottom: rows - 1,
            scroll_top: 0,
            tabs: BTreeSet::new(),
            saved_y: 0,
            saved_x: 0,
            saved_cur_attr_data: AttributeData::default(),
            saved_charset: None,
            saved_charsets: Vec::new(),
            saved_glevel: 0,
            saved_origin_mode: false,
            saved_wraparound_mode: true,
            markers: Rc::new(RefCell::new(Vec::new())),
            host_dimensions: Rc::new(BufferDimensions::new(cols, rows)),
            options,
            cols,
            rows,
            has_scrollback,
            is_clearing: Rc::new(Cell::new(false)),
            cleanup_position: 0,
            cleanup_pending: false,
            disposed: false,
            cleanup_events: Rc::new(Emitter::new()),
            cleanup_attached: Rc::new(Cell::new(false)),
        };
        buffer
            .lines
            .set_max_length(buffer.correct_buffer_length(rows)?)?;
        buffer.setup_tab_stops(None)?;
        Ok(buffer)
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn rows(&self) -> usize {
        self.rows
    }
    pub fn has_scrollback(&self) -> bool {
        self.has_scrollback && self.lines.max_length() > self.rows
    }
    pub fn is_cursor_in_viewport(&self) -> bool {
        self.ybase
            .checked_add(self.y)
            .is_some_and(|absolute| absolute >= self.ydisp && absolute - self.ydisp < self.rows)
    }
    /// Owned scratch-cell result replaces JS's reused mutable object. Extended attributes still share identity.
    pub fn get_null_cell(&self, attr: Option<&AttributeData>) -> CellData {
        self.scratch_cell(attr, 0, 1)
    }
    pub fn get_whitespace_cell(&self, attr: Option<&AttributeData>) -> CellData {
        self.scratch_cell(attr, 32, 1)
    }
    fn scratch_cell(&self, attr: Option<&AttributeData>, cp: u32, width: u32) -> CellData {
        CellData {
            content: cp | (width << super::constants::content::WIDTH_SHIFT),
            attributes: attr.map_or_else(AttributeData::default, |a| AttributeData {
                fg: a.fg,
                bg: a.bg,
                extended: a.extended.share(),
            }),
            combined_data: Vec::new(),
        }
    }
    pub fn get_blank_line(&self, attr: &AttributeData, is_wrapped: bool) -> LineRef {
        Rc::new(RefCell::new(BufferLine::new(
            self.host_dimensions.cols.get(),
            Some(&self.get_null_cell(Some(attr))),
            is_wrapped,
        )))
    }
    pub fn fill_viewport_rows(&mut self, attr: Option<&AttributeData>) -> Result<(), BufferError> {
        shared::fill_viewport_rows_owned(self, attr)
    }
    pub fn clear(&mut self) -> Result<(), BufferError> {
        self.ydisp = 0;
        self.ybase = 0;
        self.y = 0;
        self.x = 0;
        self.lines = CircularList::new(self.correct_buffer_length(self.rows)?)?;
        self.scroll_top = 0;
        self.scroll_bottom = self.rows - 1;
        self.setup_tab_stops(None)
    }
    pub fn set_wrapped(&self, row: usize, value: bool) -> Result<(), BufferError> {
        if let Some(line) = self.lines.get(row) {
            line.try_borrow_mut()
                .map_err(|_| BufferError::BorrowedLine)?
                .is_wrapped = value;
        }
        Ok(())
    }
    pub fn translate_buffer_line_to_string(
        &self,
        index: usize,
        trim_right: bool,
        start: Option<usize>,
        end: Option<usize>,
    ) -> Result<Vec<u16>, BufferError> {
        let Some(line) = self.lines.get(index) else {
            return Ok(Vec::new());
        };
        let result = line
            .try_borrow()
            .map_err(|_| BufferError::BorrowedLine)?
            .translate_to_string(trim_right, start, end, None);
        Ok(result)
    }
    pub(crate) fn line(&self, index: usize) -> Result<LineRef, BufferError> {
        self.lines.get(index).ok_or(BufferError::MissingLine(index))
    }
    /// Run one host-scheduled idle cleanup batch. True requests another batch.
    pub fn poll_cleanup(&mut self) -> Result<bool, BufferError> {
        if !self.cleanup_pending {
            return Ok(false);
        }
        let normal_run = self.cleanup_position < self.lines.len();
        if !normal_run {
            self.cleanup_position = 0;
        }
        let mut cleaned = 0;
        while self.cleanup_position < self.lines.len() {
            let index = self.cleanup_position;
            // Source get(position++) advances even when the row is undefined.
            self.cleanup_position += 1;
            let line = self.line(index)?;
            cleaned += usize::from(
                line.try_borrow_mut()
                    .map_err(|_| BufferError::BorrowedLine)?
                    .cleanup_memory(),
            );
            if cleaned > 100 {
                return Ok(true);
            }
        }
        self.cleanup_pending = normal_run;
        Ok(normal_run)
    }
    pub fn has_pending_cleanup(&self) -> bool {
        self.cleanup_pending
    }
    pub fn dispose(&mut self) -> Result<(), BufferError> {
        if self.disposed {
            return Ok(());
        }
        self.disposed = true;
        self.cleanup_pending = false;
        self.cleanup_events.fire(&false)?;
        self.clear_all_markers()
    }
}
