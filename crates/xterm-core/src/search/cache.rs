// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{SearchBuffer, SearchError};
use std::collections::BTreeMap;

pub const CACHE_TTL_MS: u64 = 15_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineCacheEntry {
    pub text: Vec<u16>,
    pub offsets: Vec<usize>,
}

/// The adapter supplies a monotonic clock and timer instead of requiring a
/// browser event loop or a background thread in the portable terminal core.
#[derive(Debug, Default)]
pub struct SearchLineCache {
    entries: Option<BTreeMap<isize, LineCacheEntry>>,
    last_access: u64,
}

impl SearchLineCache {
    pub fn init(&mut self, now_ms: u64) {
        self.expire(now_ms);
        self.entries.get_or_insert_with(BTreeMap::new);
        self.last_access = now_ms;
    }
    pub fn get(&self, row: isize) -> Option<&LineCacheEntry> {
        self.entries.as_ref()?.get(&row)
    }
    pub fn set(&mut self, row: isize, entry: LineCacheEntry) {
        if let Some(entries) = &mut self.entries {
            entries.insert(row, entry);
        }
    }
    /// Call on line feed, cursor move, resize and addon disposal.
    pub fn invalidate(&mut self) {
        self.entries = None;
        self.last_access = 0;
    }
    pub fn deadline_ms(&self) -> Option<u64> {
        self.entries
            .as_ref()
            .map(|_| self.last_access.saturating_add(CACHE_TTL_MS))
    }
    pub fn expire(&mut self, now_ms: u64) {
        if self.entries.is_some() && now_ms.saturating_sub(self.last_access) >= CACHE_TTL_MS {
            self.invalidate();
        }
    }
    pub fn translate(
        buffer: SearchBuffer<'_>,
        mut row: isize,
        trim_right: bool,
    ) -> Result<LineCacheEntry, SearchError> {
        let mut text = Vec::new();
        let mut traversed = 0;
        let mut offsets = vec![0];
        while let Some(line) = buffer.line(row) {
            let next_row = row.checked_add(1).ok_or(SearchError::IndexOverflow)?;
            let next = buffer.line(next_row);
            let wraps = next.is_some_and(|l| l.is_wrapped);
            traversed += 1;
            if wraps
                && buffer
                    .ring_capacity
                    .is_some_and(|capacity| traversed >= capacity)
            {
                return Err(SearchError::WrappedTranslationCycle);
            }
            let mut string = line.translate_to_string(!wraps && trim_right, None, None, None);
            if wraps {
                let last = line.len().checked_sub(1);
                let null_last = last.is_some_and(|col| {
                    line.get_code_point(col) == Some(0) && line.get_width(col) == Some(1)
                });
                if null_last && next.is_some_and(|l| l.get_width(0) == Some(2)) {
                    string.pop();
                }
            }
            text.extend(string);
            if !wraps {
                break;
            }
            offsets.push(text.len());
            row = next_row;
        }
        Ok(LineCacheEntry { text, offsets })
    }
}
