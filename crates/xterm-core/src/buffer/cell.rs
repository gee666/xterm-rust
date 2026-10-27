// Copyright (c) 2018, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::{constants::content, AttributeData};
use crate::input::text_decoder::string_from_code_point;
use std::ops::{Deref, DerefMut};

/// Legacy CharData tuple represented by named fields. Strings retain UTF-16
/// code units, including lone surrogates. `None` represents JS NaN in `code`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CharData {
    pub attr: u32,
    pub chars: Vec<u16>,
    pub width: i32,
    pub code: Option<u32>,
}

/// A packed terminal cell. Deref exposes its inherited AttributeData operations.
#[derive(Clone, Debug, Default)]
pub struct CellData {
    pub content: u32,
    pub attributes: AttributeData,
    pub combined_data: Vec<u16>,
}

impl CellData {
    pub fn from_char_data(value: &CharData) -> Self {
        let mut cell = Self::default();
        cell.set_from_char_data(value);
        cell
    }

    pub fn is_combined(&self) -> u32 {
        self.content & content::IS_COMBINED_MASK
    }

    pub fn get_width(&self) -> i32 {
        // Source uses arithmetic right shift, even for raw nonstandard widths.
        (self.content as i32) >> content::WIDTH_SHIFT
    }

    pub fn get_chars(&self) -> Vec<u16> {
        if self.is_combined() != 0 {
            return self.combined_data.clone();
        }
        let code = self.content & content::CODEPOINT_MASK;
        if code == 0 {
            Vec::new()
        } else {
            string_from_code_point(code)
        }
    }

    /// Combined strings return their last UTF-16 code unit, not last scalar.
    pub fn get_code(&self) -> Option<u32> {
        if self.is_combined() != 0 {
            self.combined_data.last().map(|&c| u32::from(c))
        } else {
            Some(self.content & content::CODEPOINT_MASK)
        }
    }

    /// The legacy input code is ignored, as in upstream. Empty strings yield
    /// codepoint zero through JavaScript's NaN-to-int32 bitwise conversion.
    pub fn set_from_char_data(&mut self, value: &CharData) {
        self.attributes.fg = value.attr;
        self.attributes.bg = 0;
        let width = (value.width as u32) << content::WIDTH_SHIFT;
        let code = match value.chars.as_slice() {
            [] => Some(0),
            [first] => Some(u32::from(*first)),
            [first @ 0xd800..=0xdbff, second @ 0xdc00..=0xdfff] => {
                Some((u32::from(*first) - 0xd800) * 0x400 + u32::from(*second) - 0xdc00 + 0x10000)
            }
            _ => None,
        };
        self.content = width | code.unwrap_or(content::IS_COMBINED_MASK);
        if code.is_none() {
            self.combined_data.clone_from(&value.chars);
        }
    }

    pub fn get_as_char_data(&self) -> CharData {
        CharData {
            attr: self.attributes.fg,
            chars: self.get_chars(),
            width: self.get_width(),
            code: self.get_code(),
        }
    }

    /// Compare only upstream's observable rendering attributes. Protection,
    /// hyperlink identity, underline variant offset and content are excluded.
    pub fn attributes_equals(&self, other: &Self) -> bool {
        let a = &self.attributes;
        let b = &other.attributes;
        if a.get_fg_color_mode() != b.get_fg_color_mode()
            || a.get_fg_color() != b.get_fg_color()
            || a.get_bg_color_mode() != b.get_bg_color_mode()
            || a.get_bg_color() != b.get_bg_color()
            || a.is_inverse() != b.is_inverse()
            || a.is_bold() != b.is_bold()
            || a.is_underline() != b.is_underline()
        {
            return false;
        }
        if a.is_underline() != 0 {
            if a.get_underline_style() != b.get_underline_style() {
                return false;
            }
            let a_default = a.is_underline_color_default();
            let b_default = b.is_underline_color_default();
            if !(a_default && b_default)
                && (a_default != b_default
                    || a.get_underline_color() != b.get_underline_color()
                    || a.get_underline_color_mode() != b.get_underline_color_mode())
            {
                return false;
            }
        }
        a.is_overline() == b.is_overline()
            && a.is_blink() == b.is_blink()
            && a.is_invisible() == b.is_invisible()
            && a.is_italic() == b.is_italic()
            && a.is_dim() == b.is_dim()
            && a.is_strikethrough() == b.is_strikethrough()
    }
}

impl Deref for CellData {
    type Target = AttributeData;
    fn deref(&self) -> &Self::Target {
        &self.attributes
    }
}

impl DerefMut for CellData {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.attributes
    }
}
