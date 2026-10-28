// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Search addon coordination. The host supplies selection, DOM styles and timers;
//! terminal cells, marker lifetimes and decorations remain Rust implementations.
mod addon;
mod decorations;
mod tracker;
pub use addon::{SearchAddon, SearchOptionsHandle};
pub use decorations::{DecorationManager, MultiHighlight};
pub use tracker::{SearchResultChange, SearchResultTracker};

use std::{any::Any, error::Error, fmt, rc::Rc};
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

/// Engine flags and browser colors are separate because the portable engine
/// does not own rendering state. The JS adapter presents the original flat object.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SearchOptions {
    pub matching: xterm_core::search::SearchOptions,
    pub decorations: Option<SearchDecorationOptions>,
}

#[derive(Debug)]
pub enum SearchAddonError {
    NotLoaded,
    Borrowed,
    InvalidMarker,
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
            Self::InvalidMarker => f.write_str("search match marker is no longer in the buffer"),
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
impl Error for SearchAddonError {}
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
