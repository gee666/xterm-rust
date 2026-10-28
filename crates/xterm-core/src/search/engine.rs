// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{
    coordinates::{buffer_cols_to_string_offset, string_length_to_buffer_size},
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
            let end = if self.buffer.ring_capacity.is_some() {
                start_row
            } else {
                start_row.min(self.buffer.end)
            };
            result = self.scan(term, 0..end, 0, options, false)?;
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
            let end = if self.buffer.ring_capacity.is_some() {
                start_row
            } else {
                start_row.min(self.buffer.end)
            };
            for row in (0..end).rev() {
                position.row = row;
                result = self.find_in_line(term, &mut position, options, true)?;
                if result.is_some() {
                    break;
                }
                if self.buffer.ring_capacity.is_some() && row as f64 - 1.0 == row as f64 {
                    return Err(SearchError::RowTraversalCycle);
                }
            }
        }
        if result.is_none() && start_row != last_row {
            let first = if self.buffer.ring_capacity.is_some() {
                start_row
            } else {
                start_row.max(0)
            };
            for row in (first..self.buffer.end).rev() {
                position.row = row;
                result = self.find_in_line(term, &mut position, options, true)?;
                if result.is_some() {
                    break;
                }
                if self.buffer.ring_capacity.is_some() && row as f64 - 1.0 == row as f64 {
                    return Err(SearchError::RowTraversalCycle);
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
        let first = if self.buffer.ring_capacity.is_some() {
            rows.start
        } else {
            rows.start.max(0)
        };
        let end = if self.buffer.ring_capacity.is_some() {
            rows.end
        } else {
            rows.end.min(self.buffer.end)
        };
        for row in first..end {
            let mut position = SearchPosition { row, col };
            if let Some(found) = self.find_in_line(term, &mut position, options, reverse)? {
                return Ok(Some(found));
            }
            let number = row as f64;
            if self.buffer.ring_capacity.is_some() && number + 1.0 == number {
                return Err(SearchError::RowTraversalCycle);
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
            let row_number = position.row as f64;
            if !reverse && (self.buffer.ring_capacity == Some(1) || row_number - 1.0 == row_number)
            {
                return Err(SearchError::WrappedLineRecursion);
            }
            position.col = position
                .col
                .checked_add(self.buffer.cols as isize)
                .ok_or(SearchError::IndexOverflow)?;
            if reverse {
                return Ok(None);
            }
            let previous = row_number - 1.0;
            if previous < isize::MIN as f64 || previous >= -(isize::MIN as f64) {
                return Err(SearchError::IndexOverflow);
            }
            position.row = previous as isize;
        }
        let row = position.row;
        if self.cache.get(row).is_none() {
            self.cache
                .set(row, SearchLineCache::translate(self.buffer, row, true)?);
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
        let offset = buffer_cols_to_string_offset(self.buffer, row, position.col)?;
        let Some((index, matched)) = matching::find(term, &entry.text, offset, options, reverse)?
        else {
            return Ok(None);
        };
        let match_end = index
            .checked_add(matched.len())
            .ok_or(SearchError::IndexOverflow)?;
        let mut start_row = 0;
        while start_row < entry.offsets.len() - 1 && index >= entry.offsets[start_row + 1] {
            start_row += 1;
        }
        let mut end_row = start_row;
        while end_row < entry.offsets.len() - 1 && match_end >= entry.offsets[end_row + 1] {
            end_row += 1;
        }
        let first_row = row
            .checked_add(start_row as isize)
            .ok_or(SearchError::IndexOverflow)?;
        let last_row = row
            .checked_add(end_row as isize)
            .ok_or(SearchError::IndexOverflow)?;
        let start =
            string_length_to_buffer_size(self.buffer, first_row, index - entry.offsets[start_row])?;
        let end = string_length_to_buffer_size(
            self.buffer,
            last_row,
            match_end - entry.offsets[end_row],
        )?;
        let span = self
            .buffer
            .cols
            .checked_mul(end_row - start_row)
            .and_then(|n| isize::try_from(n).ok())
            .ok_or(SearchError::IndexOverflow)?;
        let size = end
            .checked_add(span)
            .and_then(|end| end.checked_sub(start))
            .ok_or(SearchError::IndexOverflow)?;
        Ok(Some(SearchResult {
            term: matched,
            col: start,
            row: first_row,
            size,
        }))
    }
}
