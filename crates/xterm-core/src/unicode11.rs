// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of addon-unicode11/src/UnicodeV11.ts.

use crate::unicode::{
    combining_properties, in_sorted_ranges, UnicodeCharProperties, UnicodeCharWidth,
    UnicodeVersionProvider,
};
use std::sync::OnceLock;

#[path = "unicode11_tables.rs"]
mod tables;

static BMP_WIDTHS: OnceLock<[u8; 65536]> = OnceLock::new();

/// Unicode 11 width provider with the exact bundled xterm.js interval tables.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnicodeV11;

impl UnicodeV11 {
    pub fn new() -> Self {
        BMP_WIDTHS.get_or_init(build_bmp);
        Self
    }
}

impl UnicodeVersionProvider for UnicodeV11 {
    fn version(&self) -> &str {
        "11"
    }

    fn wcwidth(&self, codepoint: u32) -> UnicodeCharWidth {
        if codepoint < 32 {
            return UnicodeCharWidth::Zero;
        }
        if codepoint < 127 {
            return UnicodeCharWidth::Narrow;
        }
        if codepoint < 65536 {
            return match BMP_WIDTHS.get_or_init(build_bmp)[codepoint as usize] {
                0 => UnicodeCharWidth::Zero,
                2 => UnicodeCharWidth::Wide,
                _ => UnicodeCharWidth::Narrow,
            };
        }
        if in_sorted_ranges(tables::HIGH_COMBINING, codepoint) {
            UnicodeCharWidth::Zero
        } else if in_sorted_ranges(tables::HIGH_WIDE, codepoint) {
            UnicodeCharWidth::Wide
        } else {
            UnicodeCharWidth::Narrow
        }
    }

    fn char_properties(
        &self,
        codepoint: u32,
        preceding: UnicodeCharProperties,
    ) -> UnicodeCharProperties {
        combining_properties(self.wcwidth(codepoint), preceding)
    }
}

fn build_bmp() -> [u8; 65536] {
    let mut table = [1; 65536];
    table[..32].fill(0);
    table[0x7f..0xa0].fill(0);
    for &(start, end) in tables::BMP_COMBINING {
        table[start as usize..=end as usize].fill(0);
    }
    // Unlike V6, upstream V11 applies wide intervals after combining intervals.
    for &(start, end) in tables::BMP_WIDE {
        table[start as usize..=end as usize].fill(2);
    }
    table
}
