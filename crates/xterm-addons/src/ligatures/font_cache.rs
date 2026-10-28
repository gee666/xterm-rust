// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::{font_bytes::FontError, font_tables::GlyphValue};
use std::{cell::RefCell, rc::Rc};

pub type GlyphArray = Rc<RefCell<Vec<GlyphValue>>>;
pub type RangeArray = Rc<RefCell<Vec<[i64; 2]>>>;
pub type LigatureResult = Rc<RefCell<LigatureData>>;

pub struct LigatureData {
    pub input_glyphs: GlyphArray,
    pub output_glyphs: GlyphArray,
    pub context_ranges: RangeArray,
}
#[derive(Clone)]
pub enum Cached {
    Full(LigatureResult),
    Ranges(RangeArray),
}
impl Cached {
    pub fn ranges(&self) -> Result<RangeArray, FontError> {
        match self {
            Self::Full(data) => Ok(Rc::clone(
                &data
                    .try_borrow()
                    .map_err(|_| FontError::Borrowed)?
                    .context_ranges,
            )),
            Self::Ranges(ranges) => Ok(Rc::clone(ranges)),
        }
    }
}
struct Entry {
    key: Vec<u16>,
    value: Cached,
}
pub struct Cache {
    capacity: usize,
    size: usize,
    entries: Vec<Entry>,
}
impl Cache {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            size: 0,
            entries: Vec::new(),
        }
    }
    pub fn get(&mut self, text: &[u16]) -> Option<Cached> {
        let index = self.entries.iter().position(|entry| entry.key == text)?;
        let entry = self.entries.remove(index);
        let value = entry.value.clone();
        self.entries.push(entry);
        Some(value)
    }
    pub fn set(&mut self, text: &[u16], value: Cached) -> Result<(), FontError> {
        if text.is_empty() {
            return Err(FontError::CacheEmptyKey);
        }
        // lru-cache refuses oversized entries without rejecting the set call.
        if text.len() > self.capacity {
            return Ok(());
        }
        let mut key = Vec::new();
        key.try_reserve_exact(text.len())?;
        key.extend_from_slice(text);
        if let Some(index) = self.entries.iter().position(|entry| entry.key == text) {
            let old = self.entries.remove(index);
            self.size -= old.key.len();
        }
        while self.size > self.capacity - text.len() {
            let old = self.entries.remove(0);
            self.size -= old.key.len();
        }
        self.entries.try_reserve(1)?;
        self.size = self
            .size
            .checked_add(text.len())
            .ok_or(FontError::Capacity)?;
        self.entries.push(Entry { key, value });
        Ok(())
    }
}
