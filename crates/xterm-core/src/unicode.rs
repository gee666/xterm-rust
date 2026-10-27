// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of common/services/Services.ts.

/// Number of terminal columns occupied by a Unicode character.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum UnicodeCharWidth {
    Zero = 0,
    Narrow = 1,
    Wide = 2,
}

impl UnicodeCharWidth {
    pub const fn columns(self) -> u8 {
        self as u8
    }
}

/// Packed Unicode provider state, width and join flag, as in xterm.js.
///
/// The raw representation preserves all 32 bits, including values supplied by
/// external providers. Construction masks state to 24 bits and width to 2 bits.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UnicodeCharProperties(u32);

impl UnicodeCharProperties {
    pub const fn from_raw(value: u32) -> Self {
        Self(value)
    }

    pub const fn raw(self) -> u32 {
        self.0
    }

    pub const fn new(state: u32, width: u32, should_join: bool) -> Self {
        Self(((state & 0xffffff) << 3) | ((width & 3) << 1) | should_join as u32)
    }

    pub const fn should_join(self) -> bool {
        self.0 & 1 != 0
    }

    /// Returns the packed width, including the reserved value 3.
    pub const fn width(self) -> u8 {
        ((self.0 >> 1) & 3) as u8
    }

    /// Arithmetic right shift matches JavaScript's signed `>>` operator.
    pub const fn char_kind(self) -> i32 {
        (self.0 as i32) >> 3
    }
}

/// Shared zero-width joining behavior used by the Unicode 6 and 11 providers.
pub(crate) fn combining_properties(
    width: UnicodeCharWidth,
    preceding: UnicodeCharProperties,
) -> UnicodeCharProperties {
    let mut width = width.columns();
    let mut should_join = width == 0 && preceding.raw() != 0;
    if should_join {
        let old_width = preceding.width();
        if old_width == 0 {
            should_join = false;
        } else if old_width > width {
            width = old_width;
        }
    }
    UnicodeCharProperties::new(0, u32::from(width), should_join)
}

pub(crate) fn in_sorted_ranges(ranges: &[(u32, u32)], codepoint: u32) -> bool {
    ranges
        .binary_search_by(|&(start, end)| {
            if codepoint < start {
                std::cmp::Ordering::Greater
            } else if codepoint > end {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        })
        .is_ok()
}

/// A Unicode version's terminal width and character joining rules.
///
/// Codepoints are UTF-32 numbers rather than Rust `char`, so lone UTF-16
/// surrogates remain representable just as they are in the JavaScript API.
pub trait UnicodeVersionProvider {
    fn version(&self) -> &str;
    fn wcwidth(&self, codepoint: u32) -> UnicodeCharWidth;
    fn char_properties(
        &self,
        codepoint: u32,
        preceding: UnicodeCharProperties,
    ) -> UnicodeCharProperties;
}
