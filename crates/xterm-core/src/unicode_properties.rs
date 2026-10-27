// Copyright (c) 2023 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of addon-unicode-graphemes UnicodeProperties.ts.

//! Unicode 15 property lookup and the bundled grapheme-break rules.
//! The compressed upstream trie is generated as sorted change points instead
//! of shipping a DEFLATE implementation in the terminal engine.

#[path = "unicode15_tables.rs"]
mod tables;

pub const GRAPHEME_BREAK_MASK: u32 = 0x0f;
pub const CHARWIDTH_MASK: u32 = 0x30;
pub const CHARWIDTH_SHIFT: u32 = 4;
pub const OTHER: u32 = 0;
pub const PREPEND: u32 = 1;
pub const EXTEND: u32 = 2;
pub const REGIONAL_INDICATOR: u32 = 3;
pub const SPACING_MARK: u32 = 4;
pub const HANGUL_L: u32 = 5;
pub const HANGUL_V: u32 = 6;
pub const HANGUL_T: u32 = 7;
pub const HANGUL_LV: u32 = 8;
pub const HANGUL_LVT: u32 = 9;
pub const ZWJ: u32 = 10;
pub const EXTENDED_PICTOGRAPHIC: u32 = 11;
pub const REGIONAL_PAIR: i32 = 32;
pub const WIDTH_NORMAL: u32 = 0;
pub const WIDTH_FORCE_ONE_COLUMN: u32 = 1;
pub const WIDTH_AMBIGUOUS: u32 = 2;
pub const WIDTH_WIDE: u32 = 3;

/// Exact decoded property data. Values outside Unicode use the trie error value.
pub fn get_info(codepoint: u32) -> u32 {
    if codepoint > 0x10ffff {
        return tables::ERROR_VALUE;
    }
    let index = tables::INFO.partition_point(|&(start, _)| start <= codepoint);
    // Generated data begins at zero and covers every Unicode codepoint.
    tables::INFO[index - 1].1
}

pub fn info_to_width_info(info: u32) -> u32 {
    (info & CHARWIDTH_MASK) >> CHARWIDTH_SHIFT
}

pub fn info_to_width(info: u32, ambiguous_is_wide: bool) -> u8 {
    let width = info_to_width_info(info);
    if width >= WIDTH_AMBIGUOUS && (width >= WIDTH_WIDE || ambiguous_is_wide) {
        2
    } else {
        1
    }
}

/// Standalone widths, without grapheme joining, using UTF-16 indexing.
pub fn str_width(units: &[u16], prefer_wide: bool) -> usize {
    let mut width = 0;
    let mut index = 0;
    while index < units.len() {
        let (codepoint, length) = code_point_at(units, index);
        width += usize::from(info_to_width(get_info(codepoint), prefer_wide));
        index += length;
    }
    width
}

/// Return the UTF-16 index containing the requested column, or the end index.
/// Columns retain JS comparison behavior for negative, NaN and infinite values.
pub fn column_to_index_in_context(
    units: &[u16],
    start_index: usize,
    column: f64,
    prefer_wide: bool,
) -> usize {
    let mut index = start_index;
    let mut width = 0;
    while index < units.len() {
        let (codepoint, length) = code_point_at(units, index);
        width += usize::from(info_to_width(get_info(codepoint), prefer_wide));
        if width as f64 > column {
            return index;
        }
        index += length;
    }
    index
}

fn code_point_at(units: &[u16], index: usize) -> (u32, usize) {
    let first = units[index];
    if (0xd800..=0xdbff).contains(&first) {
        if let Some(&second) = units.get(index + 1) {
            if (0xdc00..=0xdfff).contains(&second) {
                return (
                    (u32::from(first) - 0xd800) * 0x400 + u32::from(second) - 0xdc00 + 0x10000,
                    2,
                );
            }
        }
    }
    (u32::from(first), 1)
}

/// Positive means join; nonpositive means break. The result carries next state.
pub fn should_join(before_state: i32, after_info: u32) -> i32 {
    let before = (before_state as u32) & GRAPHEME_BREAK_MASK;
    let after = after_info & GRAPHEME_BREAK_MASK;
    if joins(before, after) {
        if after == REGIONAL_INDICATOR {
            REGIONAL_PAIR
        } else {
            after as i32 + 16
        }
    } else {
        after as i32 - 16
    }
}

/// Reverse traversal counterpart, retaining the original regional-pair state.
pub fn should_join_backwards(before_info: u32, after_state: i32) -> i32 {
    let before = before_info & GRAPHEME_BREAK_MASK;
    let after = (after_state as u32) & GRAPHEME_BREAK_MASK;
    if joins(before, after) {
        if before == REGIONAL_INDICATOR {
            REGIONAL_PAIR
        } else {
            before as i32 + 16
        }
    } else {
        before as i32 - 16
    }
}

fn joins(before: u32, after: u32) -> bool {
    (before == HANGUL_L && matches!(after, HANGUL_L | HANGUL_V | HANGUL_LV | HANGUL_LVT))
        || (matches!(before, HANGUL_LV | HANGUL_V) && matches!(after, HANGUL_V | HANGUL_T))
        || (matches!(before, HANGUL_LVT | HANGUL_T) && after == HANGUL_T)
        || matches!(after, EXTEND | ZWJ | SPACING_MARK)
        || before == PREPEND
        || (before == ZWJ && after == EXTENDED_PICTOGRAPHIC)
        || (before == REGIONAL_INDICATOR && after == REGIONAL_INDICATOR)
}
