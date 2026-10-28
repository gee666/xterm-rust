// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from browser/ColorContrastCache.ts.
use crate::theme::ColorRef;
use std::collections::HashMap;

/// Outer Option means missing; an explicitly cached None is upstream's null.
#[derive(Default, Debug)]
pub struct ColorContrastCache {
    colors: HashMap<(u64, u64), Option<ColorRef>>,
    css: HashMap<(u64, u64), Option<String>>,
}
impl ColorContrastCache {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn set_css(&mut self, bg: impl Into<f64>, fg: impl Into<f64>, value: Option<String>) {
        self.css.insert(pair(bg, fg), value);
    }
    pub fn get_css(&self, bg: impl Into<f64>, fg: impl Into<f64>) -> Option<&Option<String>> {
        self.css.get(&pair(bg, fg))
    }
    pub fn set_color(&mut self, bg: impl Into<f64>, fg: impl Into<f64>, value: Option<ColorRef>) {
        self.colors.insert(pair(bg, fg), value);
    }
    pub fn get_color(&self, bg: impl Into<f64>, fg: impl Into<f64>) -> Option<&Option<ColorRef>> {
        self.colors.get(&pair(bg, fg))
    }
    pub fn clear(&mut self) {
        self.colors.clear();
        self.css.clear();
    }
}
fn pair(bg: impl Into<f64>, fg: impl Into<f64>) -> (u64, u64) {
    (key(bg.into()), key(fg.into()))
}
fn key(value: f64) -> u64 {
    // Number-to-property-string is injective except NaN payloads and signed zero.
    // Preserve signed attributes: -1 and 4294967295 must not alias through u32.
    if value == 0.0 {
        0
    } else if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_number_keys_do_not_alias_unsigned_attributes() {
        let mut cache = ColorContrastCache::new();
        cache.set_css(-1.0, 1.0, Some("signed".into()));
        cache.set_css(u32::MAX, 1, Some("unsigned".into()));
        assert_eq!(cache.get_css(-1.0, 1.0).unwrap().as_deref(), Some("signed"));
        assert_eq!(
            cache.get_css(u32::MAX, 1).unwrap().as_deref(),
            Some("unsigned")
        );
        cache.set_css(f64::NAN, -0.0, None);
        assert_eq!(cache.get_css(-f64::NAN, 0.0), Some(&None));
    }
}
