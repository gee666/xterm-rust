// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.

//! String accumulation with JavaScript UTF-16 length and payload limits.
//!
//! Rust strings cannot contain lone surrogates. `append_utf16` and `as_utf16`
//! preserve those values; `to_utf8` explicitly reports invalid UTF-16 instead of
//! silently replacing it. `len` maps upstream `length`, and `as_utf16` maps the
//! code units returned by upstream `toString`. Contiguous, amortized growth
//! replaces JS chunk joining without repeated concatenation or changed output.

use std::string::FromUtf16Error;

/// Accumulates UTF-16 units in insertion order.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct StringBuilder {
    units: Vec<u16>,
}

impl StringBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of UTF-16 units, not Unicode scalars or UTF-8 bytes.
    pub fn len(&self) -> usize {
        self.units.len()
    }

    pub fn is_empty(&self) -> bool {
        self.units.is_empty()
    }

    pub fn reset(&mut self) {
        self.units.clear();
    }

    pub fn append(&mut self, chunk: &str) {
        self.units.extend(chunk.encode_utf16());
    }

    pub fn append_utf16(&mut self, chunk: &[u16]) {
        self.units.extend_from_slice(chunk);
    }

    pub fn as_utf16(&self) -> &[u16] {
        &self.units
    }

    pub fn into_utf16(self) -> Vec<u16> {
        self.units
    }

    pub fn to_utf8(&self) -> Result<String, FromUtf16Error> {
        String::from_utf16(&self.units)
    }
}

/// Clears its accumulated payload whenever an append exceeds the fixed limit.
/// A breach does not permanently reject future chunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LimitedStringBuilder {
    builder: StringBuilder,
    limit: usize,
}

impl LimitedStringBuilder {
    pub fn new(limit: usize) -> Self {
        Self {
            builder: StringBuilder::new(),
            limit,
        }
    }

    pub fn len(&self) -> usize {
        self.builder.len()
    }

    pub fn is_empty(&self) -> bool {
        self.builder.is_empty()
    }

    pub fn limit(&self) -> usize {
        self.limit
    }

    pub fn reset(&mut self) {
        self.builder.reset();
    }

    /// Returns true on overflow. The entire payload, including this chunk, is
    /// discarded, matching upstream. Limits count UTF-16 units.
    pub fn append(&mut self, chunk: &str) -> bool {
        if self.reject_length(chunk.encode_utf16().count()) {
            return true;
        }
        self.builder.append(chunk);
        false
    }

    pub fn append_utf16(&mut self, chunk: &[u16]) -> bool {
        if self.reject_length(chunk.len()) {
            return true;
        }
        self.builder.append_utf16(chunk);
        false
    }

    pub fn as_utf16(&self) -> &[u16] {
        self.builder.as_utf16()
    }

    pub fn to_utf8(&self) -> Result<String, FromUtf16Error> {
        self.builder.to_utf8()
    }

    fn reject_length(&mut self, additional: usize) -> bool {
        // len <= limit after every append. Subtraction also avoids overflowing
        // usize for a caller-supplied limit near usize::MAX.
        if additional > self.limit - self.len() {
            self.reset();
            true
        } else {
            false
        }
    }
}
