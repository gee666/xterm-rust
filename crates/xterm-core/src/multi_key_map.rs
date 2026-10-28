// Copyright (c) 2022 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.

//! Multi-key maps matching JavaScript string/number property-key coercion.

use std::collections::HashMap;

/// A JavaScript property key. Numbers and their ECMAScript decimal string
/// representation address the same entry, including `-0` and `"0"`.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct PropertyKey(Vec<u16>);

impl From<&str> for PropertyKey {
    fn from(value: &str) -> Self {
        Self(value.encode_utf16().collect())
    }
}

impl From<String> for PropertyKey {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl From<&[u16]> for PropertyKey {
    fn from(value: &[u16]) -> Self {
        Self(value.to_vec())
    }
}

impl From<Vec<u16>> for PropertyKey {
    fn from(value: Vec<u16>) -> Self {
        Self(value)
    }
}

impl From<f64> for PropertyKey {
    fn from(value: f64) -> Self {
        Self::from(ryu_js::Buffer::new().format(value))
    }
}

impl From<u32> for PropertyKey {
    fn from(value: u32) -> Self {
        Self::from(f64::from(value))
    }
}

impl From<i32> for PropertyKey {
    fn from(value: i32) -> Self {
        Self::from(f64::from(value))
    }
}

/// Map indexed by two string or JavaScript-number keys.
///
/// Rust returns `None` for a missing value instead of JavaScript `undefined`.
/// Entries named `__proto__` or inherited object properties are plain entries:
/// JavaScript prototype mutation is deliberately not exposed in Rust.
pub struct TwoKeyMap<V> {
    data: HashMap<[PropertyKey; 2], V>,
}

impl<V> Default for TwoKeyMap<V> {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<V> TwoKeyMap<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, first: impl Into<PropertyKey>, second: impl Into<PropertyKey>, value: V) {
        self.data.insert([first.into(), second.into()], value);
    }

    pub fn get(&self, first: impl Into<PropertyKey>, second: impl Into<PropertyKey>) -> Option<&V> {
        self.data.get(&[first.into(), second.into()])
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

/// Four-key counterpart to [`TwoKeyMap`], with identical coercion rules.
pub struct FourKeyMap<V> {
    data: HashMap<[PropertyKey; 4], V>,
}

impl<V> Default for FourKeyMap<V> {
    fn default() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl<V> FourKeyMap<V> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(
        &mut self,
        first: impl Into<PropertyKey>,
        second: impl Into<PropertyKey>,
        third: impl Into<PropertyKey>,
        fourth: impl Into<PropertyKey>,
        value: V,
    ) {
        self.data.insert(
            [first.into(), second.into(), third.into(), fourth.into()],
            value,
        );
    }

    pub fn get(
        &self,
        first: impl Into<PropertyKey>,
        second: impl Into<PropertyKey>,
        third: impl Into<PropertyKey>,
        fourth: impl Into<PropertyKey>,
    ) -> Option<&V> {
        self.data
            .get(&[first.into(), second.into(), third.into(), fourth.into()])
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}
