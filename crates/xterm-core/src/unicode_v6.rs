// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of common/input/UnicodeV6.ts.

use std::sync::OnceLock;

use crate::unicode::{
    combining_properties, in_sorted_ranges, UnicodeCharProperties, UnicodeCharWidth,
    UnicodeVersionProvider,
};

#[path = "unicode_v6_tables.rs"]
mod tables;
use tables::{BMP_COMBINING, HIGH_COMBINING};

static BMP_WIDTHS: OnceLock<[u8; 65536]> = OnceLock::new();

/// xterm.js's default Unicode 6 width provider.
#[derive(Clone, Copy, Debug, Default)]
pub struct UnicodeV6;

impl UnicodeV6 {
    pub fn new() -> Self {
        BMP_WIDTHS.get_or_init(build_bmp_widths);
        Self
    }
}

impl UnicodeVersionProvider for UnicodeV6 {
    fn version(&self) -> &str {
        "6"
    }

    fn wcwidth(&self, codepoint: u32) -> UnicodeCharWidth {
        if codepoint < 32 {
            return UnicodeCharWidth::Zero;
        }
        if codepoint < 127 {
            return UnicodeCharWidth::Narrow;
        }
        if codepoint < 65536 {
            // The range check ensures this conversion is lossless on supported targets.
            return match BMP_WIDTHS.get_or_init(build_bmp_widths)[codepoint as usize] {
                0 => UnicodeCharWidth::Zero,
                2 => UnicodeCharWidth::Wide,
                _ => UnicodeCharWidth::Narrow,
            };
        }
        if in_sorted_ranges(HIGH_COMBINING, codepoint) {
            return UnicodeCharWidth::Zero;
        }
        if (0x20000..=0x2fffd).contains(&codepoint) || (0x30000..=0x3fffd).contains(&codepoint) {
            return UnicodeCharWidth::Wide;
        }
        UnicodeCharWidth::Narrow
    }

    fn char_properties(
        &self,
        codepoint: u32,
        preceding: UnicodeCharProperties,
    ) -> UnicodeCharProperties {
        combining_properties(self.wcwidth(codepoint), preceding)
    }
}

fn build_bmp_widths() -> [u8; 65536] {
    let mut table = [1; 65536];
    table[..32].fill(0);
    table[0x7f..0xa0].fill(0);
    for (start, end) in [
        (0x1100, 0x1160),
        (0x2329, 0x232b),
        (0x2e80, 0xa4d0),
        (0xac00, 0xd7a4),
        (0xf900, 0xfb00),
        (0xfe10, 0xfe1a),
        (0xfe30, 0xfe70),
        (0xff00, 0xff61),
        (0xffe0, 0xffe7),
    ] {
        table[start..end].fill(2);
    }
    table[0x303f] = 1;
    // Combining ranges must override wide ranges, including U+302A..U+302F.
    for &(start, end) in BMP_COMBINING {
        table[start as usize..=end as usize].fill(0);
    }
    table
}
