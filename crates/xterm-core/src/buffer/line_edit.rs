// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::{BufferLine, CellData};

impl BufferLine {
    /// Insert cells and repair wide characters split at either boundary.
    pub fn insert_cells(&mut self, pos: usize, count: usize, fill: &CellData) {
        self.invalidate();
        if self.is_empty() {
            return;
        }
        let pos = pos % self.length;
        if pos > 0 && self.get_width(pos - 1) == Some(2) {
            self.set_cell_from_codepoint(pos - 1, 0, 1, fill);
        }
        let count = count.min(self.length - pos);
        for i in (pos..self.length - count).rev() {
            self.copy_within_cell(i, i + count);
        }
        for i in pos..pos + count {
            self.set_cell(i, fill);
        }
        if self.get_width(self.length - 1) == Some(2) {
            self.set_cell_from_codepoint(self.length - 1, 0, 1, fill);
        }
    }
    /// Delete cells, preserving the source's directional overlap behavior.
    pub fn delete_cells(&mut self, pos: usize, count: usize, fill: &CellData) {
        self.invalidate();
        if self.is_empty() {
            return;
        }
        let pos = pos % self.length;
        let count = count.min(self.length - pos);
        for i in pos..self.length - count {
            self.copy_within_cell(i + count, i);
        }
        for i in self.length - count..self.length {
            self.set_cell(i, fill);
        }
        if pos > 0 && self.get_width(pos - 1) == Some(2) {
            self.set_cell_from_codepoint(pos - 1, 0, 1, fill);
        }
        if self.get_width(pos) == Some(0) && self.has_content(pos) == Some(0) {
            self.set_cell_from_codepoint(pos, 0, 1, fill);
        }
    }
    pub fn replace_cells(
        &mut self,
        start: usize,
        end: usize,
        fill: &CellData,
        respect_protect: bool,
    ) {
        self.invalidate();
        if start > 0
            && self.get_width(start - 1) == Some(2)
            && (!respect_protect || self.is_protected(start - 1) == Some(0))
        {
            self.set_cell_from_codepoint(start - 1, 0, 1, fill);
        }
        if end > 0
            && end < self.length
            && self.get_width(end - 1) == Some(2)
            && (!respect_protect || self.is_protected(end) == Some(0))
        {
            self.set_cell_from_codepoint(end, 0, 1, fill);
        }
        for i in start..end.min(self.length) {
            if !respect_protect || self.is_protected(i) == Some(0) {
                self.set_cell(i, fill);
            }
        }
    }
    /// Copy a range from another line. Out-of-range cells are left unchanged.
    pub fn copy_cells_from(
        &mut self,
        source: &Self,
        from: usize,
        to: usize,
        count: usize,
        reverse: bool,
    ) {
        self.invalidate();
        let count = count
            .min(source.length.saturating_sub(from))
            .min(self.length.saturating_sub(to));
        if reverse {
            for i in (0..count).rev() {
                self.copy_other_cell(source, from + i, to + i);
            }
        } else {
            for i in 0..count {
                self.copy_other_cell(source, from + i, to + i);
            }
        }
    }
    /// Self-copy is separate because Rust cannot alias a line as `&mut` and `&`.
    /// Direction is observable for overlapping ranges, just as in JavaScript.
    pub fn copy_cells_within(&mut self, from: usize, to: usize, count: usize, reverse: bool) {
        self.invalidate();
        let count = count
            .min(self.length.saturating_sub(from))
            .min(self.length.saturating_sub(to));
        if reverse {
            for i in (0..count).rev() {
                self.copy_within_cell(from + i, to + i);
            }
        } else {
            for i in 0..count {
                self.copy_within_cell(from + i, to + i);
            }
        }
    }
    fn copy_other_cell(&mut self, source: &Self, from: usize, to: usize) {
        self.data[to] = source.data[from];
        self.copy_maps(source, from, to);
    }
    fn copy_within_cell(&mut self, from: usize, to: usize) {
        // get_cell clears inactive sparse fields, as loadCell does upstream.
        if let Some(cell) = self.get_cell(from) {
            self.set_cell(to, &cell);
        }
    }
}
