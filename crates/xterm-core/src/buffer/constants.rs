// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
//! Packed buffer values match `common/buffer/Constants.ts` bit for bit.

/// Packed words per cell, upstream Constants.CELL_INDICIES.
pub const CELL_WORDS: usize = 3;
pub const DEFAULT_COLOR: u32 = 0;
pub const DEFAULT_ATTR: u32 = 256;
pub const DEFAULT_EXT: u32 = 0;
pub const NULL_CELL_CHAR: &[u16] = &[];
pub const NULL_CELL_WIDTH: i32 = 1;
pub const NULL_CELL_CODE: u32 = 0;
pub const WHITESPACE_CELL_CHAR: &[u16] = &[32];
pub const WHITESPACE_CELL_WIDTH: i32 = 1;
pub const WHITESPACE_CELL_CODE: u32 = 32;

pub mod content {
    pub const CODEPOINT_MASK: u32 = 0x1f_ffff;
    pub const IS_COMBINED_MASK: u32 = 0x20_0000;
    pub const HAS_CONTENT_MASK: u32 = 0x3f_ffff;
    pub const WIDTH_MASK: u32 = 0xc0_0000;
    pub const WIDTH_SHIFT: u32 = 22;
}

pub mod attributes {
    pub const BLUE_MASK: u32 = 0xff;
    pub const BLUE_SHIFT: u32 = 0;
    pub const PCOLOR_MASK: u32 = 0xff;
    pub const PCOLOR_SHIFT: u32 = 0;
    pub const GREEN_MASK: u32 = 0xff00;
    pub const GREEN_SHIFT: u32 = 8;
    pub const RED_MASK: u32 = 0xff_0000;
    pub const RED_SHIFT: u32 = 16;
    pub const CM_MASK: u32 = 0x300_0000;
    pub const CM_DEFAULT: u32 = 0;
    pub const CM_P16: u32 = 0x100_0000;
    pub const CM_P256: u32 = 0x200_0000;
    pub const CM_RGB: u32 = 0x300_0000;
    pub const RGB_MASK: u32 = 0xff_ffff;
}

pub mod fg_flags {
    pub const INVERSE: u32 = 0x400_0000;
    pub const BOLD: u32 = 0x800_0000;
    pub const UNDERLINE: u32 = 0x1000_0000;
    pub const BLINK: u32 = 0x2000_0000;
    pub const INVISIBLE: u32 = 0x4000_0000;
    pub const STRIKETHROUGH: u32 = 0x8000_0000;
}

pub mod bg_flags {
    pub const ITALIC: u32 = 0x400_0000;
    pub const DIM: u32 = 0x800_0000;
    pub const HAS_EXTENDED: u32 = 0x1000_0000;
    pub const PROTECTED: u32 = 0x2000_0000;
    pub const OVERLINE: u32 = 0x4000_0000;
}

pub mod ext_flags {
    pub const UNDERLINE_STYLE: u32 = 0x1c00_0000;
    pub const VARIANT_OFFSET: u32 = 0xe000_0000;
}

/// Values 6 and 7 remain representable in packed data, as upstream permits them.
pub mod underline_style {
    pub const NONE: u32 = 0;
    pub const SINGLE: u32 = 1;
    pub const DOUBLE: u32 = 2;
    pub const CURLY: u32 = 3;
    pub const DOTTED: u32 = 4;
    pub const DASHED: u32 = 5;
}
