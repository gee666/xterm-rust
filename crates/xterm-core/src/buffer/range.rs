// Copyright (c) 2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from common/buffer/BufferRange.ts.

//! Inclusive buffer coordinates. Negative coordinates are retained because the
//! upstream helper validates row ordering only, not bounds or column ordering.

use std::{error::Error, fmt};

/// JavaScript-number coordinates, including fractional and non-finite values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BufferPosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BufferRange {
    pub start: BufferPosition,
    pub end: BufferPosition,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeError {
    pub range: BufferRange,
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BufferRange { start, end } = self.range;
        let mut end_x = ryu_js::Buffer::new();
        let mut end_y = ryu_js::Buffer::new();
        let mut start_x = ryu_js::Buffer::new();
        let mut start_y = ryu_js::Buffer::new();
        write!(
            f,
            "Buffer range end ({}, {}) cannot be before start ({}, {})",
            end_x.format(end.x),
            end_y.format(end.y),
            start_x.format(start.x),
            start_y.format(start.y)
        )
    }
}

impl Error for RangeError {}

/// Return the inclusive range length, or the source error for reversed rows.
///
/// The result uses binary64 arithmetic, matching JavaScript even when multiplying
/// large coordinates loses integer precision. Equal-row reversed columns can
/// produce zero or negative lengths, and columns beyond `buffer_cols` are allowed.
pub fn get_range_length(
    range: BufferRange,
    buffer_cols: impl Into<f64>,
) -> Result<f64, RangeError> {
    if range.start.y > range.end.y {
        return Err(RangeError { range });
    }
    Ok(buffer_cols.into() * (range.end.y - range.start.y) + (range.end.x - range.start.x + 1.0))
}
