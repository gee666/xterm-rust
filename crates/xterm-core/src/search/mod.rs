// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Portable search algorithms over borrowed terminal cells.
//!
//! Positions and terms use UTF-16, not Rust byte indices. Terminal subscriptions,
//! selection mutation and browser decorations belong to the addon adapter.
mod cache;
mod engine;
mod matching;
mod state;

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
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions => f.write_str("invalid search buffer dimensions"),
            Self::InvalidColumn { col, cols } => {
                write!(f, "Invalid col: {col} to search in terminal of {cols} cols")
            }
            Self::InvalidRegex(message) => write!(f, "Invalid regular expression: {message}"),
            Self::InvalidCacheEntry => {
                f.write_str("search cache offsets must start at zero and be ordered within text")
            }
        }
    }
}
impl Error for SearchError {}

/// Immutable view of the active buffer, including its scrollback and blank rows.
/// Construct a fresh view after terminal mutations and invalidate the cache on
/// line feed, cursor movement or resize, as described in `docs/search.md`.
#[derive(Clone, Copy)]
enum Lines<'a> {
    Slice(&'a [BufferLine]),
    References(&'a [&'a BufferLine]),
}

#[derive(Clone, Copy)]
pub struct SearchBuffer<'a> {
    lines: Lines<'a>,
    cols: usize,
    end: isize,
}

impl<'a> SearchBuffer<'a> {
    pub fn new(lines: &'a [BufferLine], cols: usize) -> Result<Self, SearchError> {
        let end = Self::validate(lines.len(), cols, lines.iter().all(|l| l.len() == cols))?;
        Ok(Self {
            lines: Lines::Slice(lines),
            cols,
            end,
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
        let index = usize::try_from(row).ok()?;
        match self.lines {
            Lines::Slice(lines) => lines.get(index),
            Lines::References(lines) => lines.get(index).copied(),
        }
    }
}
