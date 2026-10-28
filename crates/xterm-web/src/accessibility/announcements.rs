// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of AccessibilityManager announcement transitions.
use std::collections::{TryReserveError, VecDeque};
use std::fmt;

pub const MAX_ROWS_TO_READ: usize = 20;

#[derive(Debug)]
pub enum AnnouncementError {
    Allocation(TryReserveError),
    LengthOverflow,
}
impl fmt::Display for AnnouncementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Allocation(error) => error.fmt(f),
            Self::LengthOverflow => f.write_str("announcement length overflow"),
        }
    }
}
impl std::error::Error for AnnouncementError {}
impl From<TryReserveError> for AnnouncementError {
    fn from(error: TryReserveError) -> Self {
        Self::Allocation(error)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharacterAction {
    Unchanged,
    Overflow,
}

/// The live DOM text is supplied to `flush`, not cached here. A user or assistive
/// technology may alter that node between events, and upstream reads it live.
#[derive(Default, Debug)]
pub struct Announcements {
    consume: VecDeque<Vec<u16>>,
    pending: Vec<u16>,
    line_count: usize,
}
impl Announcements {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn pending(&self) -> &[u16] {
        &self.pending
    }
    pub fn pending_snapshot(&self) -> Result<Vec<u16>, AnnouncementError> {
        copy(&self.pending)
    }
    pub fn clear_pending(&mut self) {
        self.pending.clear();
    }
    pub fn queued_keys(&self) -> &VecDeque<Vec<u16>> {
        &self.consume
    }
    pub fn line_count(&self) -> usize {
        self.line_count
    }

    /// Clearing the live region resets only its line count. Pending output and
    /// queued key echo survive, matching key and blur handling in the source.
    pub fn clear_live_region(&mut self) {
        self.line_count = 0;
    }

    pub fn key(&mut self, key: &[u16]) -> Result<(), AnnouncementError> {
        self.clear_live_region();
        // Unicode General_Category=Control is exactly these two BMP ranges.
        if !key
            .iter()
            .any(|unit| matches!(unit, 0..=0x1f | 0x7f..=0x9f))
        {
            let key = copy(key)?;
            self.consume.try_reserve(1)?;
            self.consume.push_back(key);
        }
        Ok(())
    }

    /// Requests immediate localized live text replacement at newline21. Other
    /// output is accumulated until a row-render flush. Each input consumes one
    /// queued key whether it matches or not; mismatches are still announced.
    pub fn character(&mut self, text: &[u16]) -> Result<CharacterAction, AnnouncementError> {
        if self.line_count > MAX_ROWS_TO_READ {
            return Ok(CharacterAction::Unchanged);
        }
        let append = self.consume.front().is_none_or(|key| key != text);
        if append {
            self.pending.try_reserve(text.len())?;
        }
        let limit = text == [10] && self.line_count == MAX_ROWS_TO_READ;
        self.consume.pop_front();
        if append {
            self.pending.extend_from_slice(text);
        }
        if text == [10] {
            self.line_count += 1;
        }
        Ok(if limit {
            CharacterAction::Overflow
        } else {
            CharacterAction::Unchanged
        })
    }

    /// Successful text-only flush for native clients. The DOM adapter performs
    /// the same commits in phases because getters/setters can throw or reenter.
    /// None means no write. Compare to the CURRENT localized label, not a cached
    /// one: changing localization during overflow affects whether count resets.
    pub fn flush(
        &mut self,
        live: &[u16],
        overflow: &[u16],
    ) -> Result<Option<Vec<u16>>, AnnouncementError> {
        if self.pending.is_empty() {
            return Ok(None);
        }
        let clear = live == overflow;
        let prefix = if clear { &[][..] } else { live };
        let size = prefix
            .len()
            .checked_add(self.pending.len())
            .ok_or(AnnouncementError::LengthOverflow)?;
        let mut result = Vec::new();
        result.try_reserve(size)?;
        result.extend_from_slice(prefix);
        result.extend_from_slice(&self.pending);
        if clear {
            self.clear_live_region();
        }
        self.clear_pending();
        Ok(Some(result))
    }
}
fn copy(value: &[u16]) -> Result<Vec<u16>, AnnouncementError> {
    let mut result = Vec::new();
    result.try_reserve(value.len())?;
    result.extend_from_slice(value);
    Ok(result)
}
