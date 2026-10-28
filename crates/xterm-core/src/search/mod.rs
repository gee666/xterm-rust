// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Portable search algorithms over borrowed terminal cells.
//!
//! Positions and terms use UTF-16, not Rust byte indices. Terminal subscriptions,
//! selection mutation and browser decorations belong to the addon adapter.
mod cache;
mod coordinates;
mod engine;
mod matching;
mod ring;
mod state;
pub use ring::with_buffer;

pub use cache::{LineCacheEntry, SearchLineCache, CACHE_TTL_MS};
pub use engine::SearchEngine;
pub use state::SearchState;

use crate::buffer::BufferLine;
use std::{error::Error, fmt};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions {
    pub case_sensitive: Option<bool>,
    pub regex: Option<bool>,
    pub whole_word: Option<bool>,
    pub incremental: Option<bool>,
    /// Presence of decoration options, independent of their browser colors.
    pub decorations: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchResult {
    pub term: Vec<u16>,
    pub col: isize,
    pub row: isize,
    pub size: isize,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct SearchPosition {
    pub row: isize,
    pub col: isize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchSelection {
    pub start: SearchPosition,
    pub end: SearchPosition,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SearchError {
    InvalidDimensions,
    InvalidColumn { col: isize, cols: usize },
    InvalidRegex(String),
    InvalidCacheEntry,
    WrappedLineRecursion,
    WrappedTranslationCycle,
    RowTraversalCycle,
    IndexOverflow,
    InvalidCellData,
    BorrowedLine,
    Storage(crate::circular_list::CircularListError),
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => f.write_str("invalid search buffer dimensions"),
            Self::InvalidColumn { col, cols } => {
                write!(f, "Invalid col: {col} to search in terminal of {cols} cols")
            }
            Self::InvalidRegex(message) => write!(f, "Invalid regular expression: {message}"),
            Self::WrappedLineRecursion => f.write_str("Maximum call stack size exceeded"),
            Self::WrappedTranslationCycle => {
                f.write_str("Wrapped search translation would traverse the same ring indefinitely")
            }
            Self::RowTraversalCycle => f.write_str(
                "search row traversal would not progress under JavaScript Number arithmetic",
            ),
            Self::IndexOverflow => {
                f.write_str("search coordinate exceeds the native signed index range")
            }
            Self::InvalidCellData => {
                f.write_str("search buffer cell has incomplete UTF-16 metadata")
            }
            Self::BorrowedLine => f.write_str("search buffer line is already borrowed"),
            Self::Storage(error) => error.fmt(f),
            Self::InvalidCacheEntry => {
                f.write_str("search cache offsets must start at zero and be ordered within text")
            }
        }
    }
}
impl Error for SearchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Storage(error) => Some(error),
            _ => None,
        }
    }
}

/// Immutable view of the active buffer, including its scrollback and blank rows.
/// Construct a fresh view after terminal mutations and invalidate the cache on
/// line feed, cursor movement or resize, as described in `docs/search.md`.
#[derive(Clone, Copy)]
enum Lines<'a> {
    Slice(&'a [BufferLine]),
    References(&'a [&'a BufferLine]),
    Lookup(&'a dyn Fn(isize) -> Option<&'a BufferLine>),
}

#[derive(Clone, Copy)]
pub struct SearchBuffer<'a> {
    lines: Lines<'a>,
    cols: usize,
    end: isize,
    ring_capacity: Option<usize>,
}

impl<'a> SearchBuffer<'a> {
    pub fn new(lines: &'a [BufferLine], cols: usize) -> Result<Self, SearchError> {
        let end = Self::validate(lines.len(), cols, lines.iter().all(|l| l.len() == cols))?;
        Ok(Self {
            lines: Lines::Slice(lines),
            cols,
            end,
            ring_capacity: None,
        })
    }
    /// Use references obtained from live ring-buffer borrow guards without
    /// copying cells. Keep those guards alive for the entire synchronous search.
    pub fn from_rows(lines: &'a [&'a BufferLine], cols: usize) -> Result<Self, SearchError> {
        let end = Self::validate(lines.len(), cols, lines.iter().all(|l| l.len() == cols))?;
        Ok(Self {
            lines: Lines::References(lines),
            cols,
            end,
            ring_capacity: None,
        })
    }
    /// The source public view delegates signed indices to its ring, including
    /// negative and beyond-length reads. The host holds immutable guards for all
    /// allocated slots while the lookup is used. Capacity bounds forward cycles.
    pub fn from_ring_lookup(
        length: usize,
        cols: usize,
        capacity: usize,
        lookup: &'a dyn Fn(isize) -> Option<&'a BufferLine>,
    ) -> Result<Self, SearchError> {
        let end = Self::validate(length, cols, true)?;
        // Retained ring slots can have old widths after alternate-buffer resize.
        // Source BufferApiView exposes them; per-cell access handles short rows.
        if capacity == 0 {
            return Err(SearchError::InvalidDimensions);
        }
        Ok(Self {
            lines: Lines::Lookup(lookup),
            cols,
            end,
            ring_capacity: Some(capacity),
        })
    }
    fn validate(len: usize, cols: usize, uniform: bool) -> Result<isize, SearchError> {
        let end = isize::try_from(len).map_err(|_| SearchError::InvalidDimensions)?;
        if cols == 0
            || !uniform
            || cols
                .checked_mul(len.max(1))
                .is_none_or(|n| isize::try_from(n).is_err())
        {
            return Err(SearchError::InvalidDimensions);
        }
        Ok(end)
    }
    pub fn cols(&self) -> usize {
        self.cols
    }
    pub fn len(&self) -> usize {
        self.end as usize
    }
    pub fn is_empty(&self) -> bool {
        self.end == 0
    }
    pub fn line(&self, row: isize) -> Option<&'a BufferLine> {
        match self.lines {
            Lines::Slice(lines) => usize::try_from(row).ok().and_then(|index| lines.get(index)),
            Lines::References(lines) => usize::try_from(row)
                .ok()
                .and_then(|index| lines.get(index).copied()),
            Lines::Lookup(lookup) => lookup(row),
        }
    }
}
