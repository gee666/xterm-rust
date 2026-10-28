// Copyright (c) 2016, 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. See LICENSE.

//! VT100 maps retain mutable object identity. A print run captures the handle,
//! not its contents, so edits made by a callback affect the next lookup.
use std::{
    cell::{BorrowError, BorrowMutError, RefCell},
    collections::{BTreeMap, TryReserveError},
    error::Error,
    fmt,
    rc::Rc,
};

#[path = "charset_tables.rs"]
mod tables;

/// An explicit undefined property differs from a missing property in snapshots.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CharsetValue {
    Undefined,
    Text(Vec<u16>),
}

/// Shared UTF-16 map. Cloning shares identity; reads return owned data or a unit.
/// No borrow guard escapes a public operation or crosses a terminal callback.
#[derive(Clone, Debug, Default)]
pub struct Charset {
    entries: Rc<RefCell<BTreeMap<u16, CharsetValue>>>,
}
impl Charset {
    pub fn new(entries: BTreeMap<u16, Vec<u16>>) -> Self {
        Self {
            entries: Rc::new(RefCell::new(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, CharsetValue::Text(value)))
                    .collect(),
            )),
        }
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.entries, &other.entries)
    }
    /// Empty replacements and undefined values leave printing unchanged.
    pub fn get_first(&self, key: u16) -> Result<Option<u16>, CharsetError> {
        let entries = self.entries.try_borrow().map_err(CharsetError::Read)?;
        Ok(match entries.get(&key) {
            Some(CharsetValue::Text(value)) => value.first().copied(),
            _ => None,
        })
    }
    pub fn get(&self, key: u16) -> Result<Option<Vec<u16>>, CharsetError> {
        let entries = self.entries.try_borrow().map_err(CharsetError::Read)?;
        Ok(match entries.get(&key) {
            Some(CharsetValue::Text(value)) => Some(value.clone()),
            _ => None,
        })
    }
    /// `None` assigns an own undefined property; use `remove` to delete it.
    pub fn set(&self, key: u16, value: Option<Vec<u16>>) -> Result<(), CharsetError> {
        self.entries
            .try_borrow_mut()
            .map_err(CharsetError::Write)?
            .insert(
                key,
                value.map_or(CharsetValue::Undefined, CharsetValue::Text),
            );
        Ok(())
    }
    /// Returns whether a property existed, unlike JavaScript's delete operator.
    pub fn remove(&self, key: u16) -> Result<bool, CharsetError> {
        Ok(self
            .entries
            .try_borrow_mut()
            .map_err(CharsetError::Write)?
            .remove(&key)
            .is_some())
    }
    pub fn snapshot(&self) -> Result<BTreeMap<u16, CharsetValue>, CharsetError> {
        Ok(self
            .entries
            .try_borrow()
            .map_err(CharsetError::Read)?
            .clone())
    }
}

pub const DEFAULT_CHARSET: Option<Charset> = None;

/// Aliases and repeated lookups share objects within this thread's realm.
/// The core is !Send, like a JS terminal bound to a realm. A thread-local cache
/// preserves shared mutable built-ins without imposing global locks or leaking
/// allocations. A different native thread, like a JS worker, has its own maps.
pub fn builtin_charset(designator: char) -> Option<Charset> {
    thread_local! {
        static CHARSETS: BTreeMap<char, Charset> = tables::build_charsets();
    }
    CHARSETS.with(|charsets| charsets.get(&designator).cloned())
}

#[derive(Debug)]
pub enum CharsetError {
    LevelOverflow,
    Allocation(TryReserveError),
    Read(BorrowError),
    Write(BorrowMutError),
}
impl fmt::Display for CharsetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LevelOverflow => f.write_str("charset index exceeds the array length range"),
            Self::Allocation(error) => write!(f, "cannot allocate designated charsets: {error}"),
            Self::Read(error) => write!(f, "cannot read charset map: {error}"),
            Self::Write(error) => write!(f, "cannot modify charset map: {error}"),
        }
    }
}
impl Error for CharsetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LevelOverflow => None,
            Self::Allocation(error) => Some(error),
            Self::Read(error) => Some(error),
            Self::Write(error) => Some(error),
        }
    }
}

/// Levels are nonnegative indices; missing slots correspond to JS array holes.
#[derive(Clone, Debug, Default)]
pub struct CharsetService {
    pub charset: Option<Charset>,
    pub glevel: usize,
    charsets: Vec<Option<Charset>>,
}
impl CharsetService {
    pub fn charsets(&self) -> &[Option<Charset>] {
        &self.charsets
    }
    /// Direct array edits do not refresh the separately selected active map.
    pub fn charsets_mut(&mut self) -> &mut Vec<Option<Charset>> {
        &mut self.charsets
    }
    pub fn reset(&mut self) {
        self.charset = None;
        self.charsets.clear();
        self.glevel = 0;
    }
    pub fn set_g_level(&mut self, g: usize) {
        self.glevel = g;
        self.charset = self.charsets.get(g).and_then(Clone::clone);
    }
    /// Failed growth does not change any part of the service.
    pub fn set_g_charset(
        &mut self,
        g: usize,
        charset: Option<Charset>,
    ) -> Result<(), CharsetError> {
        let len = g.checked_add(1).ok_or(CharsetError::LevelOverflow)?;
        if len > self.charsets.len() {
            self.charsets
                .try_reserve(len - self.charsets.len())
                .map_err(CharsetError::Allocation)?;
            self.charsets.resize_with(len, || None);
        }
        self.charsets[g] = charset.clone();
        if self.glevel == g {
            self.charset = charset;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn borrows_are_reported_and_never_escape_lookup_or_set() {
        let map = Charset::new(BTreeMap::from([(u16::from(b'q'), vec![0x2500])]));
        {
            let _held = map.entries.borrow_mut();
            assert!(matches!(
                map.get_first(u16::from(b'q')),
                Err(CharsetError::Read(_))
            ));
            assert!(matches!(map.set(0, None), Err(CharsetError::Write(_))));
        }
        assert_eq!(map.get_first(u16::from(b'q')).unwrap(), Some(0x2500));
        map.set(u16::from(b'q'), Some(vec![0x41])).unwrap();
        assert_eq!(map.get_first(u16::from(b'q')).unwrap(), Some(0x41));
    }
}
