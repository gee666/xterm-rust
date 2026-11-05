// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Keep fractional JavaScript addresses out of the native usize cell interface.
use super::BufferLine;
use crate::buffer::{constants::CELL_WORDS, ExtendedAttrs};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

/// Retains an extended-attribute property map across reentrant key conversion.
/// In-place deletion/insertion stays visible; fill/copy replacement does not.
pub struct ExtendedAttributesRead {
    map: Rc<RefCell<BTreeMap<usize, ExtendedAttrs>>>,
}
impl ExtendedAttributesRead {
    pub fn get(&self, index: usize) -> Option<ExtendedAttrs> {
        self.map.borrow().get(&index).map(ExtendedAttrs::share)
    }
}

fn array_index(value: f64, length: usize) -> Option<usize> {
    if !value.is_finite() || value < 0.0 || value.fract() != 0.0 || value >= length as f64 {
        return None;
    }
    Some(value as usize)
}

impl BufferLine {
    /// TypedArray reads use the packed word address, not a rounded cell index.
    pub fn numeric_word(&self, address: f64) -> Option<u32> {
        // The source's TypedArray view shrinks before its backing allocation.
        // Retained capacity must never become observable through numeric reads.
        let index = array_index(address, self.length * CELL_WORDS)?;
        self.data
            .get(index / CELL_WORDS)
            .map(|row| row[index % CELL_WORDS])
    }

    /// Capture the current property object before invoking a user key hook.
    pub fn prepare_extended_read(&self) -> ExtendedAttributesRead {
        ExtendedAttributesRead {
            map: Rc::clone(&self.extended),
        }
    }

    pub(crate) fn numeric_combined(&self, index: f64) -> Option<Vec<u16>> {
        self.combined
            .get(&array_index(index, self.length)?)
            .cloned()
    }

    pub(crate) fn numeric_extended(&self, index: f64) -> Option<ExtendedAttrs> {
        self.extended
            .borrow()
            .get(&array_index(index, self.length)?)
            .map(ExtendedAttrs::share)
    }
}
