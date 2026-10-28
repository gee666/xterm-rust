// Copyright (c) 2019-2026 The xterm.js authors. All rights reserved. MIT license.
//! Browser link algorithms with explicit host boundaries for URL validation and callbacks.
#[cfg(target_arch = "wasm32")]
pub mod animation_queue;
#[cfg(target_arch = "wasm32")]
pub mod browser;
#[cfg(target_arch = "wasm32")]
pub mod browser_debounce;
#[cfg(target_arch = "wasm32")]
pub mod browser_linkifier;
pub mod debounce;
#[cfg(target_arch = "wasm32")]
pub mod dom_helpers;
mod hover;
#[cfg(target_arch = "wasm32")]
pub mod native_osc;
mod osc;
mod provider;
mod replies;
pub mod time_debounce;
#[cfg(target_arch = "wasm32")]
pub mod window_interval;

pub use hover::{Decorations, HoverEffect, HoverState};
pub use osc::{osc_links, try_osc_links, try_osc_links_with_policy, OscBuffer};
pub use provider::{LinkProviderService, ProviderRegistration};
pub use replies::{ProviderReplies, ProviderRequest, ReplyGroup};

/// Buffer cell coordinates are one-based, including inclusive link end positions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// Generated endpoints can be one cell beyond a 32-bit indexed row/window.
/// Keep them independent of pointer-sized indices, especially on WASM32.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangePosition {
    pub x: u64,
    pub y: u64,
}
impl From<Position> for RangePosition {
    fn from(position: Position) -> Self {
        Self {
            x: position.x as u64,
            y: position.y as u64,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Range {
    pub start: RangePosition,
    pub end: RangePosition,
}

impl Range {
    /// Matches the source's row-major comparison, even for positions outside a row.
    pub fn contains(self, position: Position, cols: usize) -> bool {
        let linear = |p: RangePosition| p.y as u128 * cols as u128 + p.x as u128;
        linear(self.start) <= linear(position.into()) && linear(position.into()) <= linear(self.end)
    }
}

/// Link data is separate from host event callbacks, so native tests need no DOM.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Link {
    pub text: Vec<u16>,
    pub range: Range,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnderlineEvent {
    pub x1: i64,
    pub y1: i64,
    pub x2: u64,
    pub y2: i64,
    pub cols: usize,
}

impl Link {
    /// Convert buffer coordinates to zero-based viewport drawing coordinates.
    pub fn underline_event(&self, cols: usize, scroll_offset: usize) -> UnderlineEvent {
        UnderlineEvent {
            x1: self.range.start.x as i64 - 1,
            y1: self.range.start.y as i64 - scroll_offset as i64 - 1,
            x2: self.range.end.x,
            y2: self.range.end.y as i64 - scroll_offset as i64 - 1,
            cols,
        }
    }
}
