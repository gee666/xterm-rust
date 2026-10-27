// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::{
    constants::{attributes as color, bg_flags as bg, fg_flags as fg, underline_style},
    ExtendedAttrs,
};

/// Foreground/background bit patterns use u32 instead of JS signed bitwise numbers.
/// Flag queries return their mask, not a normalized boolean, matching upstream.
#[derive(Clone, Debug, Default)]
pub struct AttributeData {
    pub fg: u32,
    pub bg: u32,
    pub extended: ExtendedAttrs,
}

impl AttributeData {
    pub fn to_color_rgb(value: u32) -> [u8; 3] {
        [
            (value >> color::RED_SHIFT) as u8,
            (value >> color::GREEN_SHIFT) as u8,
            value as u8,
        ]
    }

    pub fn from_color_rgb(value: [u32; 3]) -> u32 {
        ((value[0] & 255) << color::RED_SHIFT)
            | ((value[1] & 255) << color::GREEN_SHIFT)
            | (value[2] & 255)
    }

    pub fn is_inverse(&self) -> u32 {
        self.fg & fg::INVERSE
    }
    pub fn is_bold(&self) -> u32 {
        self.fg & fg::BOLD
    }
    pub fn is_underline(&self) -> u32 {
        if self.has_extended_attrs() != 0
            && self.extended.underline_style() != underline_style::NONE
        {
            1
        } else {
            self.fg & fg::UNDERLINE
        }
    }
    pub fn is_blink(&self) -> u32 {
        self.fg & fg::BLINK
    }
    pub fn is_invisible(&self) -> u32 {
        self.fg & fg::INVISIBLE
    }
    pub fn is_italic(&self) -> u32 {
        self.bg & bg::ITALIC
    }
    pub fn is_dim(&self) -> u32 {
        self.bg & bg::DIM
    }
    pub fn is_strikethrough(&self) -> u32 {
        self.fg & fg::STRIKETHROUGH
    }
    pub fn is_protected(&self) -> u32 {
        self.bg & bg::PROTECTED
    }
    pub fn is_overline(&self) -> u32 {
        self.bg & bg::OVERLINE
    }
    pub fn get_fg_color_mode(&self) -> u32 {
        self.fg & color::CM_MASK
    }
    pub fn get_bg_color_mode(&self) -> u32 {
        self.bg & color::CM_MASK
    }
    pub fn is_fg_rgb(&self) -> bool {
        self.get_fg_color_mode() == color::CM_RGB
    }
    pub fn is_bg_rgb(&self) -> bool {
        self.get_bg_color_mode() == color::CM_RGB
    }
    pub fn is_fg_palette(&self) -> bool {
        is_palette(self.fg)
    }
    pub fn is_bg_palette(&self) -> bool {
        is_palette(self.bg)
    }
    pub fn is_fg_default(&self) -> bool {
        self.get_fg_color_mode() == 0
    }
    pub fn is_bg_default(&self) -> bool {
        self.get_bg_color_mode() == 0
    }
    pub fn is_attribute_default(&self) -> bool {
        self.fg == 0 && self.bg == 0
    }
    pub fn get_fg_color(&self) -> i32 {
        get_color(self.fg)
    }
    pub fn get_bg_color(&self) -> i32 {
        get_color(self.bg)
    }
    pub fn has_extended_attrs(&self) -> u32 {
        self.bg & bg::HAS_EXTENDED
    }

    pub fn update_extended(&mut self) {
        if self.extended.is_empty() {
            self.bg &= !bg::HAS_EXTENDED;
        } else {
            self.bg |= bg::HAS_EXTENDED;
        }
    }

    pub fn get_underline_color(&self) -> i32 {
        if self.has_extended_attrs() != 0 {
            let value = self.extended.underline_color();
            if value & color::CM_MASK != color::CM_DEFAULT {
                return get_color(value);
            }
        }
        self.get_fg_color()
    }

    pub fn get_underline_color_mode(&self) -> u32 {
        if self.has_extended_attrs() != 0 {
            self.extended.underline_color() & color::CM_MASK
        } else {
            self.get_fg_color_mode()
        }
    }
    pub fn is_underline_color_rgb(&self) -> bool {
        self.get_underline_color_mode() == color::CM_RGB
    }
    pub fn is_underline_color_palette(&self) -> bool {
        is_palette(self.get_underline_color_mode())
    }
    pub fn is_underline_color_default(&self) -> bool {
        self.get_underline_color_mode() == color::CM_DEFAULT
    }
    pub fn get_underline_style(&self) -> u32 {
        if self.fg & fg::UNDERLINE == 0 {
            underline_style::NONE
        } else if self.has_extended_attrs() != 0 {
            self.extended.underline_style()
        } else {
            underline_style::SINGLE
        }
    }
    pub fn get_underline_variant_offset(&self) -> u32 {
        self.extended.underline_variant_offset()
    }
}

fn get_color(value: u32) -> i32 {
    match value & color::CM_MASK {
        color::CM_P16 | color::CM_P256 => (value & color::PCOLOR_MASK) as i32,
        color::CM_RGB => (value & color::RGB_MASK) as i32,
        _ => -1,
    }
}

fn is_palette(value: u32) -> bool {
    matches!(value & color::CM_MASK, color::CM_P16 | color::CM_P256)
}
