// Copyright (c) 2023 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of UnicodeGraphemeProvider.ts.

use crate::unicode::{UnicodeCharProperties, UnicodeCharWidth, UnicodeVersionProvider};
use crate::unicode_properties as uc;
use std::cell::Cell;

/// Unicode 15 provider, optionally applying the bundled grapheme joining rules.
/// Ambiguous character width remains mutable after provider registration.
#[derive(Debug)]
pub struct UnicodeGraphemeProvider {
    pub ambiguous_chars_are_wide: Cell<bool>,
    handle_graphemes: bool,
}

impl Default for UnicodeGraphemeProvider {
    fn default() -> Self {
        Self::new(true)
    }
}

impl UnicodeGraphemeProvider {
    pub fn new(handle_graphemes: bool) -> Self {
        Self {
            ambiguous_chars_are_wide: Cell::new(false),
            handle_graphemes,
        }
    }

    pub fn handle_graphemes(&self) -> bool {
        self.handle_graphemes
    }
}

impl UnicodeVersionProvider for UnicodeGraphemeProvider {
    fn version(&self) -> &str {
        if self.handle_graphemes {
            "15-graphemes"
        } else {
            "15"
        }
    }

    fn wcwidth(&self, codepoint: u32) -> UnicodeCharWidth {
        let info = uc::get_info(codepoint);
        let width = uc::info_to_width_info(info);
        let kind = info & uc::GRAPHEME_BREAK_MASK;
        if matches!(kind, uc::EXTEND | uc::PREPEND) {
            UnicodeCharWidth::Zero
        } else if width >= uc::WIDTH_AMBIGUOUS
            && (width == uc::WIDTH_WIDE || self.ambiguous_chars_are_wide.get())
        {
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
        if (32..127).contains(&codepoint) && preceding.char_kind() == 0 {
            return UnicodeCharProperties::new(uc::OTHER, 1, false);
        }
        let mut info = uc::get_info(codepoint) as i32;
        let width_info = uc::info_to_width_info(info as u32);
        // The source charProperties starts at one even for Extend/Prepend.
        // Joining, rather than standalone wcwidth, accounts for their zero width.
        let mut width = if width_info >= uc::WIDTH_AMBIGUOUS
            && (width_info == uc::WIDTH_WIDE
                || self.ambiguous_chars_are_wide.get()
                || codepoint == 0xfe0f)
        {
            2
        } else {
            1
        };
        let mut should_join = false;
        if preceding.raw() != 0 {
            let old_width = preceding.width();
            info = if self.handle_graphemes {
                uc::should_join(preceding.char_kind(), info as u32)
            } else {
                // Upstream assigns width >= 1 before this branch, so its
                // `w === 0 ? 1 : 0` always chooses zero. Do not substitute wcwidth.
                0
            };
            should_join = info > 0;
            if should_join {
                if old_width > width {
                    width = old_width;
                } else if info == uc::REGIONAL_PAIR {
                    width = 2;
                }
            }
        }
        UnicodeCharProperties::new(info as u32, u32::from(width), should_join)
    }
}
