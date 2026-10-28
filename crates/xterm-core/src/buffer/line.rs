// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Packed line storage. Strings are UTF-16, including unpaired surrogates.
use super::{
    constants::{attributes, bg_flags, content},
    AttributeData, CellData, CharData, ExtendedAttrs,
};
use std::{cell::RefCell, collections::BTreeMap};

#[path = "line_edit.rs"]
mod edit;
#[cfg(test)]
#[path = "line_storage_tests.rs"]
mod storage_tests;
#[cfg(test)]
#[path = "line_buffer_cleanup_tests.rs"]
mod buffer_cleanup_tests;
#[path = "line_text.rs"]
mod text;
#[path = "line_numeric.rs"]
mod numeric;
#[path = "line_snapshot.rs"]
mod snapshot;
pub use snapshot::LineSnapshot;

/// One terminal row with sparse combined strings and extended attributes.
/// Out-of-range cell access returns `None`; mutations outside the row are ignored,
/// matching typed-array writes without exposing unchecked indexing to callers.
#[derive(Debug)]
pub struct BufferLine {
    data: Vec<[u32; 3]>,
    length: usize,
    combined: BTreeMap<usize, Vec<u16>>,
    extended: BTreeMap<usize, ExtendedAttrs>,
    pub is_wrapped: bool,
    cache: RefCell<Option<(Vec<u16>, bool)>>,
}

impl BufferLine {
    pub fn new(cols: usize, fill: Option<&CellData>, is_wrapped: bool) -> Self {
        let mut line = Self {
            data: vec![[0; 3]; cols],
            length: cols,
            combined: BTreeMap::new(),
            extended: BTreeMap::new(),
            is_wrapped,
            cache: RefCell::new(None),
        };
        let empty = CellData {
            content: 1 << content::WIDTH_SHIFT,
            ..CellData::default()
        };
        for i in 0..cols {
            line.set_cell(i, fill.unwrap_or(&empty));
        }
        line
    }
    pub fn len(&self) -> usize {
        self.length
    }
    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
    fn invalidate(&mut self) {
        *self.cache.get_mut() = None;
    }
    fn raw(&self, index: usize) -> Option<[u32; 3]> {
        if index < self.length {
            Some(self.data[index])
        } else {
            None
        }
    }
    pub fn get_width(&self, index: usize) -> Option<i32> {
        self.raw(index)
            .map(|c| (c[0] as i32) >> content::WIDTH_SHIFT)
    }
    pub fn has_width(&self, index: usize) -> Option<u32> {
        self.raw(index).map(|c| c[0] & content::WIDTH_MASK)
    }
    pub fn get_fg(&self, index: usize) -> Option<u32> {
        self.raw(index).map(|c| c[1])
    }
    pub fn get_bg(&self, index: usize) -> Option<u32> {
        self.raw(index).map(|c| c[2])
    }
    pub fn has_content(&self, index: usize) -> Option<u32> {
        self.raw(index).map(|c| c[0] & content::HAS_CONTENT_MASK)
    }
    pub fn is_combined(&self, index: usize) -> Option<u32> {
        self.raw(index).map(|c| c[0] & content::IS_COMBINED_MASK)
    }
    pub fn is_protected(&self, index: usize) -> Option<u32> {
        self.get_bg(index).map(|bg| bg & bg_flags::PROTECTED)
    }
    pub fn get_code_point(&self, index: usize) -> Option<u32> {
        let c = self.raw(index)?[0];
        if c & content::IS_COMBINED_MASK != 0 {
            self.combined.get(&index)?.last().copied().map(u32::from)
        } else {
            Some(c & content::CODEPOINT_MASK)
        }
    }
    pub fn get_string(&self, index: usize) -> Option<Vec<u16>> {
        let c = self.raw(index)?[0];
        if c & content::IS_COMBINED_MASK != 0 {
            return self.combined.get(&index).cloned();
        }
        let cp = c & content::CODEPOINT_MASK;
        Some(if cp == 0 {
            Vec::new()
        } else {
            text::codepoint(cp)
        })
    }
    pub fn get_extended(&self, index: usize) -> Option<ExtendedAttrs> {
        let bg = self.get_bg(index)?;
        if bg & bg_flags::HAS_EXTENDED != 0 {
            self.extended.get(&index).map(copy_extended)
        } else {
            Some(ExtendedAttrs::default())
        }
    }
    pub fn get_cell(&self, index: usize) -> Option<CellData> {
        let [c, fg, bg] = self.raw(index)?;
        Some(CellData {
            content: c,
            attributes: AttributeData {
                fg,
                bg,
                extended: self.get_extended(index)?,
            },
            combined_data: if c & content::IS_COMBINED_MASK != 0 {
                self.combined.get(&index)?.clone()
            } else {
                Vec::new()
            },
        })
    }
    pub fn load_cell<'a>(&self, index: usize, cell: &'a mut CellData) -> Option<&'a mut CellData> {
        *cell = self.get_cell(index)?;
        Some(cell)
    }
    pub fn get(&self, index: usize) -> Option<CharData> {
        Some(self.get_cell(index)?.get_as_char_data())
    }
    /// Legacy setter treats every multi-unit string as combined, even surrogate pairs.
    /// Unlike `CellData::set_from_char_data`, it preserves the previous background.
    pub fn set(&mut self, index: usize, value: &CharData) {
        self.invalidate();
        if index >= self.length {
            return;
        }
        let code = if value.chars.len() > 1 {
            self.combined.insert(index, value.chars.clone());
            index as u32 | content::IS_COMBINED_MASK
        } else {
            value.chars.first().copied().map(u32::from).unwrap_or(0)
        };
        self.data[index][0] = code | ((value.width as u32) << content::WIDTH_SHIFT);
        self.data[index][1] = value.attr;
    }
    pub fn set_cell(&mut self, index: usize, cell: &CellData) {
        self.invalidate();
        if index >= self.length {
            return;
        }
        if cell.content & content::IS_COMBINED_MASK != 0 {
            self.combined.insert(index, cell.combined_data.clone());
        }
        if cell.bg & bg_flags::HAS_EXTENDED != 0 {
            self.extended.insert(index, copy_extended(&cell.extended));
        }
        self.data[index] = [cell.content, cell.fg, cell.bg];
    }
    pub fn set_cell_from_codepoint(
        &mut self,
        index: usize,
        cp: u32,
        width: i32,
        attrs: &AttributeData,
    ) {
        self.invalidate();
        if index >= self.length {
            return;
        }
        if attrs.bg & bg_flags::HAS_EXTENDED != 0 {
            self.extended.insert(index, copy_extended(&attrs.extended));
        }
        self.data[index] = [
            cp | ((width as u32) << content::WIDTH_SHIFT),
            attrs.fg,
            attrs.bg,
        ];
    }
    pub fn add_codepoint_to_cell(&mut self, index: usize, cp: u32, width: i32) {
        self.invalidate();
        let Some([mut c, _, _]) = self.raw(index) else {
            return;
        };
        if c & content::IS_COMBINED_MASK != 0 {
            if let Some(s) = self.combined.get_mut(&index) {
                s.extend(text::codepoint(cp));
            }
        } else if c & content::CODEPOINT_MASK != 0 {
            let mut s = text::codepoint(c & content::CODEPOINT_MASK);
            s.extend(text::codepoint(cp));
            self.combined.insert(index, s);
            c = (c & !content::CODEPOINT_MASK) | content::IS_COMBINED_MASK;
        } else {
            c = cp | (1 << content::WIDTH_SHIFT);
        }
        if width != 0 {
            c = (c & !content::WIDTH_MASK) | ((width as u32) << content::WIDTH_SHIFT);
        }
        self.data[index][0] = c;
    }
    pub fn resize(&mut self, cols: usize, fill: &CellData) -> bool {
        self.invalidate();
        let old = self.length;
        if cols > self.data.len() {
            self.data.resize(cols, [0; 3]);
        }
        self.length = cols;
        if cols > old {
            for i in old..cols {
                self.set_cell(i, fill);
            }
        } else {
            self.combined.retain(|&i, _| i < cols);
            self.extended.retain(|&i, _| i < cols);
        }
        self.needs_cleanup()
    }
    fn needs_cleanup(&self) -> bool {
        self.length < self.data.len().div_ceil(2)
    }
    pub fn cleanup_memory(&mut self) -> bool {
        if !self.needs_cleanup() {
            return false;
        }
        self.data.truncate(self.length);
        self.data.shrink_to_fit();
        true
    }
    pub fn fill(&mut self, fill: &CellData, respect_protect: bool) {
        self.invalidate();
        if !respect_protect {
            self.combined.clear();
            self.extended.clear();
        }
        for i in 0..self.length {
            if !respect_protect || self.is_protected(i) == Some(0) {
                self.set_cell(i, fill);
            }
        }
    }
    pub fn copy_from(&mut self, source: &Self, blank: bool) {
        self.invalidate();
        if self.length != source.length {
            self.data = source.data[..source.length].to_vec();
        } else {
            self.data[..self.length].copy_from_slice(&source.data[..source.length]);
        }
        self.length = source.length;
        self.is_wrapped = source.is_wrapped;
        self.combined.clear();
        self.extended.clear();
        if !blank {
            for i in 0..source.length {
                self.copy_maps(source, i, i);
            }
        }
    }
    pub fn clone_line(&self, blank: bool) -> Self {
        let mut result = Self::new(0, None, false);
        result.copy_from(self, blank);
        result
    }
    fn copy_maps(&mut self, src: &Self, from: usize, to: usize) {
        if src.data[from][0] & content::IS_COMBINED_MASK != 0 {
            if let Some(s) = src.combined.get(&from) {
                self.combined.insert(to, s.clone());
            }
        }
        if src.data[from][2] & bg_flags::HAS_EXTENDED != 0 {
            if let Some(e) = src.extended.get(&from) {
                self.extended.insert(to, copy_extended(e));
            }
        }
    }
    pub fn get_trimmed_length(&self) -> usize {
        self.trimmed(false)
    }
    pub fn get_no_bg_trimmed_length(&self) -> usize {
        self.trimmed(true)
    }
    fn trimmed(&self, background: bool) -> usize {
        for i in (0..self.length).rev() {
            let [c, _, bg] = self.data[i];
            if c & content::HAS_CONTENT_MASK != 0 || (background && bg & attributes::CM_MASK != 0) {
                return i.saturating_add(((c as i32) >> content::WIDTH_SHIFT).max(0) as usize);
            }
        }
        0
    }
}

// Assignment shares the complete mutable JS object. Explicit clone() remains
// a separate operation on ExtendedAttrs and intentionally drops payload.
fn copy_extended(source: &ExtendedAttrs) -> ExtendedAttrs {
    source.share()
}

impl Clone for BufferLine {
    fn clone(&self) -> Self {
        self.clone_line(false)
    }
}
