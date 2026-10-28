// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Search addon coordination. The host supplies selection, DOM styles and timers;
//! terminal cells, marker lifetimes and decorations remain Rust implementations.
mod addon;
mod decorations;
mod tracker;
pub use addon::{SearchAddon, SearchOptionsHandle, SearchOptionsSource, WeakSearchAddon};
pub use decorations::{DecorationManager, MultiHighlight};
pub use tracker::{SearchResultChange, SearchResultTracker};

use std::{any::Any, cell::RefCell, error::Error, fmt, rc::Rc};
use xterm_core::{
    decoration::DecorationService,
    event::EmitError,
    headless::{HeadlessError, Terminal},
    lifecycle::DisposableHandle,
    search::{SearchError, SearchResult, SearchSelection},
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchDecorationOptions {
    pub match_background: Option<String>,
    pub match_border: Option<String>,
    pub match_overview_ruler: String,
    pub active_match_background: Option<String>,
    pub active_match_border: Option<String>,
    pub active_match_overview_ruler: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchColor {
    MatchBackground,
    MatchBorder,
    MatchOverview,
    ActiveBackground,
    ActiveBorder,
    ActiveOverview,
}

/// Borders are read on render, not frozen when the search starts. A browser
/// adapter retains the raw JS object; Rust callers may use values or RefCell.
pub trait SearchDecorationSource {
    fn color(&self, kind: SearchColor) -> Result<Option<String>, EmitError>;
}
pub type SearchDecorationOptionsHandle = Rc<dyn SearchDecorationSource>;
impl SearchDecorationSource for SearchDecorationOptions {
    fn color(&self, kind: SearchColor) -> Result<Option<String>, EmitError> {
        Ok(match kind {
            SearchColor::MatchBackground => self.match_background.clone(),
            SearchColor::MatchBorder => self.match_border.clone(),
            SearchColor::MatchOverview => Some(self.match_overview_ruler.clone()),
            SearchColor::ActiveBackground => self.active_match_background.clone(),
            SearchColor::ActiveBorder => self.active_match_border.clone(),
            SearchColor::ActiveOverview => Some(self.active_match_overview_ruler.clone()),
        })
    }
}
impl SearchDecorationSource for RefCell<SearchDecorationOptions> {
    fn color(&self, kind: SearchColor) -> Result<Option<String>, EmitError> {
        self.try_borrow()
            .map_err(|_| EmitError::callback(SearchAddonError::Borrowed))?
            .color(kind)
    }
}

/// Engine flags and browser colors are separate because the portable engine
/// does not own rendering state. The JS adapter presents the original flat object.
#[derive(Clone, Default)]
pub struct SearchOptions {
    pub matching: xterm_core::search::SearchOptions,
    pub decorations: Option<SearchDecorationOptionsHandle>,
}

#[derive(Debug)]
pub enum SearchAddonError {
    NotLoaded,
    Borrowed,
    InvalidCoordinates,
    Search(SearchError),
    Terminal(HeadlessError),
    Dispatch(EmitError),
    Host(Rc<dyn Error>),
}
impl fmt::Display for SearchAddonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotLoaded => f.write_str("Cannot use addon until it has been loaded"),
            Self::Borrowed => f.write_str("search terminal state is already borrowed"),
            Self::InvalidCoordinates => {
                f.write_str("search match coordinates are not representable by the host")
            }
            Self::Search(error) => error.fmt(f),
            Self::Terminal(error) => error.fmt(f),
            Self::Dispatch(error) => error.fmt(f),
            Self::Host(error) => error.fmt(f),
        }
    }
}
impl Error for SearchAddonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Search(error) => Some(error),
            Self::Terminal(error) => Some(error),
            Self::Dispatch(error) => Some(error),
            Self::Host(error) => Some(error.as_ref()),
            _ => None,
        }
    }
}
impl From<SearchError> for SearchAddonError {
    fn from(error: SearchError) -> Self {
        Self::Search(error)
    }
}
impl From<HeadlessError> for SearchAddonError {
    fn from(error: HeadlessError) -> Self {
        Self::Terminal(error)
    }
}
impl From<EmitError> for SearchAddonError {
    fn from(error: EmitError) -> Self {
        Self::Dispatch(error)
    }
}

/// The browser retains the original option value so its numeric coercion can
/// run at the same comparison/slice points, including throwing or reentering.
pub trait SearchHighlightLimit {
    fn reached(&self, count: usize) -> Result<bool, SearchAddonError>;
    fn end_index(&self, length: usize) -> Result<usize, SearchAddonError>;
}
pub struct NumericHighlightLimit(pub f64);
impl SearchHighlightLimit for NumericHighlightLimit {
    fn reached(&self, count: usize) -> Result<bool, SearchAddonError> {
        Ok(count as f64 >= self.0)
    }
    fn end_index(&self, length: usize) -> Result<usize, SearchAddonError> {
        Ok(slice_end(length, self.0))
    }
}
pub(super) fn slice_end(length: usize, maximum: f64) -> usize {
    let length = length as f64;
    let index = maximum.trunc();
    let end = if index.is_nan() {
        0.0
    } else if index < 0.0 {
        (length + index).max(0.0)
    } else {
        index.min(length)
    };
    end as usize
}

pub type TimeoutCallback = Rc<dyn Fn() -> Result<(), SearchAddonError>>;

/// These operations must act on the same terminal/selection/decoration services
/// used by normal browser input and rendering. A private search-only selection
/// or decoration command log is not an implementation of this host contract.
pub trait SearchHost {
    fn terminal(&self) -> Rc<Terminal>;
    fn decorations(&self) -> Rc<DecorationService>;
    fn selection(&self) -> Result<Option<SearchSelection>, SearchAddonError>;
    fn clear_selection(&self) -> Result<(), SearchAddonError>;
    fn select(&self, result: &SearchResult) -> Result<(), SearchAddonError>;
    fn style_decoration(
        &self,
        element: &Rc<dyn Any>,
        border: Option<&str>,
    ) -> Result<(), EmitError>;
    fn now_ms(&self) -> u64;
    fn timeout(
        &self,
        delay_ms: u64,
        callback: TimeoutCallback,
    ) -> Result<DisposableHandle, SearchAddonError>;
}
