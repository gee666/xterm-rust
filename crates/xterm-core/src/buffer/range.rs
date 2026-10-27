// Copyright (c) 2021 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from common/buffer/BufferRange.ts.

//! Inclusive buffer coordinates. Negative coordinates are retained because the
//! upstream helper validates row ordering only, not bounds or column ordering.

use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferPosition {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BufferRange {
    pub start: BufferPosition,
    pub end: BufferPosition,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RangeError {
    pub range: BufferRange,
}

impl fmt::Display for RangeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let BufferRange { start, end } = self.range;
        write!(
            f,
            "Buffer range end ({}, {}) cannot be before start ({}, {})",
            end.x, end.y, start.x, start.y
        )
    }
}

impl Error for RangeError {}

/// Return the inclusive range length, or the source error for reversed rows.
///
/// The result uses binary64 arithmetic, matching JavaScript even when multiplying
/// large coordinates loses integer precision. Equal-row reversed columns can
/// produce zero or negative lengths, and columns beyond `buffer_cols` are allowed.
pub fn get_range_length(range: BufferRange, buffer_cols: i32) -> Result<f64, RangeError> {
    if range.start.y > range.end.y {
        return Err(RangeError { range });
    }
    Ok(
        f64::from(buffer_cols) * (f64::from(range.end.y) - f64::from(range.start.y))
            + (f64::from(range.end.x) - f64::from(range.start.x) + 1.0),
    )
}
