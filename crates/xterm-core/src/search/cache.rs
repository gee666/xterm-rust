// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use super::{SearchBuffer, SearchError};
use std::{cell::OnceCell, collections::BTreeMap};

pub const CACHE_TTL_MS: u64 = 15_000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineCacheEntry {
    pub text: Vec<u16>,
    pub offsets: Vec<usize>,
}

#[derive(Debug)]
struct CachedLine {
    value: LineCacheEntry,
    lowercase: OnceCell<Vec<u16>>,
}

/// The adapter supplies a monotonic clock and timer instead of requiring a
/// browser event loop or a background thread in the portable terminal core.
#[derive(Debug, Default)]
pub struct SearchLineCache {
    entries: Option<BTreeMap<isize, CachedLine>>,
    last_access: u64,
}

impl SearchLineCache {
    pub fn init(&mut self, now_ms: u64) {
        self.expire(now_ms);
        self.entries.get_or_insert_with(BTreeMap::new);
        self.last_access = now_ms;
    }
    pub fn get(&self, row: isize) -> Option<&LineCacheEntry> {
        Some(&self.entries.as_ref()?.get(&row)?.value)
    }
    pub(super) fn lowercase(&self, row: isize) -> Option<&[u16]> {
        let entry = self.entries.as_ref()?.get(&row)?;
        // A long wrapped line can contain thousands of matches. Lowercasing
        // the same cached text for every match makes highlight-all quadratic.
        // This derived text expires with the source line, including replacement.
        Some(
            entry
                .lowercase
                .get_or_init(|| super::matching::lowercase(&entry.value.text)),
        )
    }
    pub fn set(&mut self, row: isize, entry: LineCacheEntry) {
        if let Some(entries) = &mut self.entries {
            entries.insert(
                row,
                CachedLine {
                    value: entry,
                    lowercase: OnceCell::new(),
                },
            );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(text: &[u16]) -> LineCacheEntry {
        LineCacheEntry {
            text: text.to_vec(),
            offsets: vec![0],
        }
    }

    #[test]
    fn lowercase_is_lazy_and_reuses_full_utf16_context() {
        let mut cache = SearchLineCache::default();
        cache.init(0);
        let mut text: Vec<_> = "ΟΣ İ".encode_utf16().collect();
        text.extend([0xd800, u16::from(b'A')]);
        cache.set(0, entry(&text));
        assert!(cache.entries.as_ref().unwrap()[&0]
            .lowercase
            .get()
            .is_none());
        let first = cache.lowercase(0).unwrap();
        let mut expected: Vec<_> = "ος i\u{307}".encode_utf16().collect();
        expected.extend([0xd800, u16::from(b'a')]);
        assert_eq!(first, expected);
        assert!(std::ptr::eq(first, cache.lowercase(0).unwrap()));
    }

    #[test]
    fn lowercase_is_replaced_and_expires_with_source_entry() {
        let mut cache = SearchLineCache::default();
        cache.init(0);
        cache.set(-1, entry(&[u16::from(b'A')]));
        assert_eq!(cache.lowercase(-1), Some(&[u16::from(b'a')][..]));
        cache.set(-1, entry(&[u16::from(b'B')]));
        assert_eq!(cache.lowercase(-1), Some(&[u16::from(b'b')][..]));
        cache.expire(CACHE_TTL_MS);
        assert!(cache.lowercase(-1).is_none());
        cache.init(CACHE_TTL_MS);
        cache.set(-1, entry(&[u16::from(b'C')]));
        assert_eq!(cache.lowercase(-1), Some(&[u16::from(b'c')][..]));
        cache.invalidate();
        assert!(cache.lowercase(-1).is_none());
    }
}
