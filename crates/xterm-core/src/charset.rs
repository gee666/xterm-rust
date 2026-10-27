// Copyright (c) 2016, 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. See LICENSE.

//! VT100 character maps and ISO 2022 designation state.
//!
//! Character maps use UTF-16 code units for both keys and replacements. A map
//! replacement may contain several units, such as the Dutch `ij`. Consumers
//! decide how to render that replacement; the map does not truncate it.

use std::collections::{BTreeMap, TryReserveError};
use std::error::Error;
use std::fmt;
use std::sync::{Arc, OnceLock};

#[path = "charset_tables.rs"]
mod tables;

/// Shared, immutable character map. Clones preserve map identity.
///
/// Custom maps are accepted by [`CharsetService`] in addition to the built-ins.
/// The terminal only looks up single UTF-16 units, so multi-unit JS object keys
/// have no corresponding terminal lookup and are not represented here.
pub type Charset = Arc<BTreeMap<u16, Vec<u16>>>;

/// US ASCII requires no replacement map in the upstream implementation.
pub const DEFAULT_CHARSET: Option<Charset> = None;

/// Return the map selected by an upstream `CHARSETS` designator.
///
/// `B` and unknown designators return `None`, matching JS `undefined`. Aliases
/// return the same shared map, not separately allocated equivalent maps.
pub fn builtin_charset(designator: char) -> Option<Charset> {
    static CHARSETS: OnceLock<BTreeMap<char, Charset>> = OnceLock::new();
    CHARSETS
        .get_or_init(tables::build_charsets)
        .get(&designator)
        .cloned()
}

/// Failure to grow the designated-charset array. The service remains unchanged.
#[derive(Debug)]
pub enum CharsetError {
    /// `usize::MAX` cannot be represented as an array index with a length.
    LevelOverflow,
    /// The requested array cannot be allocated.
    Allocation(TryReserveError),
}

impl fmt::Display for CharsetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::LevelOverflow => f.write_str("charset index exceeds the array length range"),
            Self::Allocation(error) => write!(f, "cannot allocate designated charsets: {error}"),
        }
    }
}

impl Error for CharsetError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::LevelOverflow => None,
            Self::Allocation(error) => Some(error),
        }
    }
}

/// Designated maps and the currently invoked G level.
///
/// Levels use nonnegative Rust indices. Protocol callers normally use 0 through
/// 3, but arbitrary array indices are retained for faithful service behavior.
/// Unassigned slots are `None`, the equivalent of JS array holes/`undefined`.
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

    /// Like the upstream array getter, direct edits do not update `charset`.
    /// Use [`Self::set_g_charset`] when designation should update the active map.
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

    /// Designate a map. Growth failures do not change any part of the service.
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
