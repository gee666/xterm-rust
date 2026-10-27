// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{
    matching, SearchBuffer, SearchError, SearchLineCache, SearchOptions, SearchPosition,
    SearchResult, SearchSelection,
};

pub struct SearchEngine<'buffer, 'cache> {
    buffer: SearchBuffer<'buffer>,
    cache: &'cache mut SearchLineCache,
    now_ms: u64,
}

impl<'buffer, 'cache> SearchEngine<'buffer, 'cache> {
    pub fn new(
        buffer: SearchBuffer<'buffer>,
        cache: &'cache mut SearchLineCache,
        now_ms: u64,
    ) -> Self {
        Self {
            buffer,
            cache,
            now_ms,
        }
    }
    pub fn find(
        &mut self,
        term: &[u16],
        row: isize,
        col: isize,
        options: &SearchOptions,
    ) -> Result<Option<SearchResult>, SearchError> {
        if term.is_empty() {
            return Ok(None);
        }
        if col >= self.buffer.cols as isize {
            return Err(SearchError::InvalidColumn {
                col,
                cols: self.buffer.cols,
            });
        }
        self.cache.init(self.now_ms);
        let mut position = SearchPosition { row, col };
        if let Some(found) = self.find_in_line(term, &mut position, options, false)? {
            return Ok(Some(found));
        }
        self.scan(
            term,
            row.saturating_add(1)..self.buffer.end,
            0,
            options,
            false,
        )
    }
    /// The terminal adapter must clear selection before applying the returned
    /// match, including when the term is empty or no match exists.
    pub fn find_next(
        &mut self,
        term: &[u16],
        options: &SearchOptions,
        cached_term: Option<&[u16]>,
        selection: Option<SearchSelection>,
    ) -> Result<Option<SearchResult>, SearchError> {
        if term.is_empty() {
            return Ok(None);
        }
        self.cache.init(self.now_ms);
        let mut position = selection.map_or(SearchPosition::default(), |s| {
            if cached_term == Some(term) {
                s.end
            } else {
                s.start
            }
        });
        let start_row = position.row;
        let mut result = self.find_in_line(term, &mut position, options, false)?;
        if result.is_none() {
            result = self.scan(
                term,
                start_row.saturating_add(1)..self.buffer.end,
                0,
                options,
                false,
            )?;
        }
        if result.is_none() && start_row != 0 {
            result = self.scan(term, 0..start_row.min(self.buffer.end), 0, options, false)?;
        }
        if result.is_none() {
            if let Some(selection) = selection {
                result = self.find_in_line(
                    term,
                    &mut SearchPosition {
                        row: selection.start.row,
                        col: 0,
                    },
                    options,
                    false,
                )?;
            }
        }
        Ok(result)
    }
    pub fn find_previous(
        &mut self,
        term: &[u16],
        options: &SearchOptions,
        cached_term: Option<&[u16]>,
        selection: Option<SearchSelection>,
    ) -> Result<Option<SearchResult>, SearchError> {
        if term.is_empty() {
            return Ok(None);
        }
        self.cache.init(self.now_ms);
        let last_row = self.buffer.end - 1;
        let mut start_row = last_row;
        let mut position = SearchPosition {
            row: start_row,
            col: self.buffer.cols as isize,
        };
        let mut result = None;
        if let Some(selection) = selection {
            position = selection.start;
            start_row = position.row;
            if cached_term != Some(term) {
                result = self.find_in_line(term, &mut position, options, false)?;
                if result.is_none() {
                    position = selection.end;
                    start_row = position.row;
                }
            }
        }
        if result.is_none() {
            result = self.find_in_line(term, &mut position, options, true)?;
        }
        if result.is_none() {
            position.col = position.col.max(self.buffer.cols as isize);
            for row in (0..start_row.min(self.buffer.end)).rev() {
                position.row = row;
                result = self.find_in_line(term, &mut position, options, true)?;
                if result.is_some() {
                    break;
                }
            }
        }
        if result.is_none() && start_row != last_row {
            for row in (start_row.max(0)..self.buffer.end).rev() {
                position.row = row;
                result = self.find_in_line(term, &mut position, options, true)?;
                if result.is_some() {
                    break;
                }
            }
        }
        Ok(result)
    }
    fn scan(
        &mut self,
        term: &[u16],
        rows: std::ops::Range<isize>,
        col: isize,
        options: &SearchOptions,
        reverse: bool,
    ) -> Result<Option<SearchResult>, SearchError> {
        for row in rows.start.max(0)..rows.end.min(self.buffer.end) {
            let mut position = SearchPosition { row, col };
            if let Some(found) = self.find_in_line(term, &mut position, options, reverse)? {
                return Ok(Some(found));
            }
        }
        Ok(None)
    }
    fn find_in_line(
        &mut self,
        term: &[u16],
        position: &mut SearchPosition,
        options: &SearchOptions,
        reverse: bool,
    ) -> Result<Option<SearchResult>, SearchError> {
        // Iteration replaces upstream recursion on arbitrarily long wrapped
        // lines, retaining the same accumulated starting column.
        while self.buffer.line(position.row).is_some_and(|l| l.is_wrapped) {
            position.col = position.col.saturating_add(self.buffer.cols as isize);
            if reverse {
                return Ok(None);
            }
            position.row = position.row.saturating_sub(1);
        }
        let row = position.row;
        if self.cache.get(row).is_none() {
            self.cache
                .set(row, SearchLineCache::translate(self.buffer, row, true));
        }
        // init always precedes searches and set inserted this row above.
        let entry = self.cache.get(row).expect("initialized search cache entry");
        if entry.offsets.first() != Some(&0)
            || entry.offsets.windows(2).any(|pair| pair[0] > pair[1])
            || entry
                .offsets
                .last()
                .is_some_and(|&end| end > entry.text.len())
        {
            return Err(SearchError::InvalidCacheEntry);
        }
        let offset = self.buffer_cols_to_string_offset(row, position.col);
        let Some((index, matched)) = matching::find(term, &entry.text, offset, options, reverse)?
        else {
            return Ok(None);
        };
        let mut start_row = 0;
        while start_row < entry.offsets.len() - 1 && index >= entry.offsets[start_row + 1] {
            start_row += 1;
        }
        let mut end_row = start_row;
        while end_row < entry.offsets.len() - 1
            && index + matched.len() >= entry.offsets[end_row + 1]
        {
            end_row += 1;
        }
        let start = self.string_length_to_buffer_size(
            row + start_row as isize,
            index - entry.offsets[start_row],
        );
        let end = self.string_length_to_buffer_size(
            row + end_row as isize,
            index + matched.len() - entry.offsets[end_row],
        );
        let size = end + (self.buffer.cols * (end_row - start_row)) as isize - start;
        Ok(Some(SearchResult {
            term: matched,
            col: start,
            row: row + start_row as isize,
            size,
        }))
    }
    fn string_length_to_buffer_size(&self, row: isize, offset: usize) -> isize {
        let mut offset = offset as isize;
        let Some(line) = self.buffer.line(row) else {
            return 0;
        };
        let mut i = 0;
        while (i as isize) < offset {
            let Some(cell) = line.get_cell(i) else { break };
            let length = cell.get_chars().len();
            if length > 1 {
                offset -= (length - 1) as isize;
            }
            if line.get_width(i + 1) == Some(0) {
                offset += 1;
            }
            i += 1;
        }
        offset
    }
    fn buffer_cols_to_string_offset(&self, mut row: isize, mut cols: isize) -> usize {
        let mut offset = 0;
        while cols > 0 {
            let Some(line) = self.buffer.line(row) else {
                break;
            };
            for i in 0..(cols as usize).min(self.buffer.cols) {
                let Some(cell) = line.get_cell(i) else { break };
                if cell.get_width() != 0 {
                    offset += if cell.get_code() == Some(0) {
                        1
                    } else {
                        cell.get_chars().len()
                    };
                }
            }
            row += 1;
            if self.buffer.line(row).is_some_and(|l| !l.is_wrapped) {
                break;
            }
            cols -= self.buffer.cols as isize;
        }
        offset
    }
}
