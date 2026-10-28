// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Keep fractional JavaScript addresses out of the native usize cell interface.
use super::BufferLine;
use crate::buffer::{constants::CELL_WORDS, ExtendedAttrs};

fn array_index(value: f64, length: usize) -> Option<usize> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value >= length as f64 {
        return None;
    }
    Some(value as usize)
}

impl BufferLine {
    /// TypedArray reads use the packed word address, not a rounded cell index.
    pub(crate) fn numeric_word(&self, address: f64) -> Option<u32> {
        let index = array_index(address, self.data.len() * CELL_WORDS)?;
        Some(self.data[index / CELL_WORDS][index % CELL_WORDS])
    }

    pub(crate) fn numeric_combined(&self, index: f64) -> Option<Vec<u16>> {
        self.combined
            .get(&array_index(index, self.length)?)
            .cloned()
    }

    pub(crate) fn numeric_extended(&self, index: f64) -> Option<ExtendedAttrs> {
        self.extended
            .get(&array_index(index, self.length)?)
            .map(ExtendedAttrs::share)
    }
}
