// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! SGR state changes ported from InputHandler.charAttributes and color helpers.

use crate::buffer::{
    constants::{attributes as color, bg_flags as bg, fg_flags as fg, underline_style},
    AttributeData,
};
use crate::parser::params::Params;

mod code {
    pub const RESET: i32 = 0;
    pub const BOLD: i32 = 1;
    pub const FAINT: i32 = 2;
    pub const ITALIC: i32 = 3;
    pub const UNDERLINE: i32 = 4;
    pub const BLINK: i32 = 5;
    pub const INVERSE: i32 = 7;
    pub const INVISIBLE: i32 = 8;
    pub const STRIKE: i32 = 9;
    pub const DOUBLE_UNDERLINE: i32 = 21;
    pub const NORMAL_INTENSITY: i32 = 22;
    pub const NO_ITALIC: i32 = 23;
    pub const NO_UNDERLINE: i32 = 24;
    pub const NO_BLINK: i32 = 25;
    pub const NO_INVERSE: i32 = 27;
    pub const NO_INVISIBLE: i32 = 28;
    pub const NO_STRIKE: i32 = 29;
    pub const FG_FIRST: i32 = 30;
    pub const FG_LAST: i32 = 37;
    pub const FG_COLOR: i32 = 38;
    pub const FG_DEFAULT: i32 = 39;
    pub const BG_FIRST: i32 = 40;
    pub const BG_LAST: i32 = 47;
    pub const BG_COLOR: i32 = 48;
    pub const BG_DEFAULT: i32 = 49;
    pub const OVERLINE: i32 = 53;
    pub const NO_OVERLINE: i32 = 55;
    pub const UNDERLINE_COLOR: i32 = 58;
    pub const UNDERLINE_DEFAULT: i32 = 59;
    pub const BRIGHT_FG_FIRST: i32 = 90;
    pub const BRIGHT_FG_LAST: i32 = 97;
    pub const BRIGHT_BG_FIRST: i32 = 100;
    pub const BRIGHT_BG_LAST: i32 = 107;
    pub const NO_BOLD: i32 = 221;
    pub const NO_FAINT: i32 = 222;
}
const RGB_MODE: i32 = 2;
const PALETTE_MODE: i32 = 5;
const COLOR_BITS: u32 = color::CM_MASK | color::RGB_MASK;

enum ColorTarget {
    Foreground,
    Background,
    Underline,
}

/// Apply SGR with the upstream default kitty bold/faint control enabled.
/// Unknown parameters are returned in source order for the terminal's debug log.
pub fn apply(attr: &mut AttributeData, params: &Params) -> Vec<i32> {
    apply_with_options(attr, params, true)
}

/// Apply SGR with the live kittySgrBoldFaintControl option supplied by the caller.
pub fn apply_with_options(
    attr: &mut AttributeData,
    params: &Params,
    kitty_control: bool,
) -> Vec<i32> {
    let mut unknown = Vec::new();
    let mut cursor = SgrCursor::new(params);
    while let Some(value) = cursor.step(attr, kitty_control) {
        unknown.push(value);
    }
    unknown
}

/// A source boundary that may call user-owned option getters or loggers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SgrAction {
    Done,
    Unknown(i32),
    Kitty(KittyControl),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KittyControl {
    Bold,
    Faint,
}

impl KittyControl {
    pub fn apply(self, attr: &mut AttributeData, enabled: bool) -> Option<i32> {
        match (self, enabled) {
            (Self::Bold, true) => attr.fg &= !fg::BOLD,
            (Self::Faint, true) => attr.bg &= !bg::DIM,
            (Self::Bold, false) => return Some(code::NO_BOLD),
            (Self::Faint, false) => return Some(code::NO_FAINT),
        }
        None
    }
}

/// Pause before each external callback and before any following mutation.
/// Bindings release their state borrow before user-owned logging/getters, then
/// resume against the same attribute object, even if reentrant reset detaches it.
pub struct SgrCursor<'a> {
    params: &'a Params,
    index: usize,
}

impl<'a> SgrCursor<'a> {
    pub fn new(params: &'a Params) -> Self {
        Self { params, index: 0 }
    }

    /// Convenience for pure attribute processing with a fixed option value.
    pub fn step(&mut self, attr: &mut AttributeData, kitty_control: bool) -> Option<i32> {
        loop {
            match self.next(attr) {
                SgrAction::Done => return None,
                SgrAction::Unknown(value) => return Some(value),
                SgrAction::Kitty(control) => {
                    if let Some(value) = control.apply(attr, kitty_control) {
                        return Some(value);
                    }
                }
            }
        }
    }

    pub fn next(&mut self, attr: &mut AttributeData) -> SgrAction {
        let params = self.params;
        while self.index < params.len() {
            let index = self.index;
            let value = params.values()[index];
            self.index += 1;
            match value {
                code::FG_FIRST..=code::FG_LAST => {
                    attr.fg =
                        (attr.fg & !COLOR_BITS) | color::CM_P16 | (value - code::FG_FIRST) as u32
                }
                code::BG_FIRST..=code::BG_LAST => {
                    attr.bg =
                        (attr.bg & !COLOR_BITS) | color::CM_P16 | (value - code::BG_FIRST) as u32
                }
                code::BRIGHT_FG_FIRST..=code::BRIGHT_FG_LAST => {
                    attr.fg = (attr.fg & !COLOR_BITS)
                        | color::CM_P16
                        | (value - code::BRIGHT_FG_FIRST) as u32
                        | 8
                }
                code::BRIGHT_BG_FIRST..=code::BRIGHT_BG_LAST => {
                    attr.bg = (attr.bg & !COLOR_BITS)
                        | color::CM_P16
                        | (value - code::BRIGHT_BG_FIRST) as u32
                        | 8
                }
                code::RESET => reset(attr),
                code::BOLD => attr.fg |= fg::BOLD,
                code::FAINT => attr.bg |= bg::DIM,
                code::ITALIC => attr.bg |= bg::ITALIC,
                code::UNDERLINE => {
                    let style = params
                        .get_sub_params(index)
                        .and_then(|sub| sub.first())
                        .copied()
                        .unwrap_or(underline_style::SINGLE as i32);
                    underline(attr, style);
                }
                code::BLINK => attr.fg |= fg::BLINK,
                code::INVERSE => attr.fg |= fg::INVERSE,
                code::INVISIBLE => attr.fg |= fg::INVISIBLE,
                code::STRIKE => attr.fg |= fg::STRIKETHROUGH,
                code::DOUBLE_UNDERLINE => underline(attr, underline_style::DOUBLE as i32),
                code::NORMAL_INTENSITY => {
                    attr.fg &= !fg::BOLD;
                    attr.bg &= !bg::DIM;
                }
                code::NO_ITALIC => attr.bg &= !bg::ITALIC,
                code::NO_UNDERLINE => underline(attr, underline_style::NONE as i32),
                code::NO_BLINK => attr.fg &= !fg::BLINK,
                code::NO_INVERSE => attr.fg &= !fg::INVERSE,
                code::NO_INVISIBLE => attr.fg &= !fg::INVISIBLE,
                code::NO_STRIKE => attr.fg &= !fg::STRIKETHROUGH,
                code::FG_DEFAULT => attr.fg &= !COLOR_BITS,
                code::BG_DEFAULT => attr.bg &= !COLOR_BITS,
                code::FG_COLOR => {
                    self.index += extract_color(attr, params, index, ColorTarget::Foreground)
                }
                code::BG_COLOR => {
                    self.index += extract_color(attr, params, index, ColorTarget::Background)
                }
                code::UNDERLINE_COLOR => {
                    self.index += extract_color(attr, params, index, ColorTarget::Underline)
                }
                code::OVERLINE => attr.bg |= bg::OVERLINE,
                code::NO_OVERLINE => attr.bg &= !bg::OVERLINE,
                code::NO_BOLD => return SgrAction::Kitty(KittyControl::Bold),
                code::NO_FAINT => return SgrAction::Kitty(KittyControl::Faint),
                code::UNDERLINE_DEFAULT => {
                    attr.extended = attr.extended.clone();
                    attr.extended.set_underline_color(u32::MAX);
                    attr.update_extended();
                }
                _ => return SgrAction::Unknown(value),
            }
        }
        SgrAction::Done
    }
}

fn reset(attr: &mut AttributeData) {
    attr.fg = color::CM_DEFAULT;
    attr.bg = color::CM_DEFAULT;
    attr.extended = attr.extended.clone();
    attr.extended.set_underline_style(underline_style::NONE);
    attr.extended
        .set_underline_color(attr.extended.underline_color() & !COLOR_BITS);
    attr.update_extended();
}

fn underline(attr: &mut AttributeData, style: i32) {
    attr.extended = attr.extended.clone();
    let style = if style == -1 || style > underline_style::DASHED as i32 {
        underline_style::SINGLE
    } else {
        style as u32
    };
    attr.extended.set_underline_style(style);
    if style == underline_style::NONE {
        attr.fg &= !fg::UNDERLINE;
    } else {
        attr.fg |= fg::UNDERLINE;
    }
    attr.update_extended();
}

fn update_color(current: u32, mode: i32, values: [i32; 3]) -> u32 {
    match mode {
        RGB_MODE => {
            (current | color::CM_RGB) & !color::RGB_MASK
                | AttributeData::from_color_rgb(values.map(|value| value as u32))
        }
        PALETTE_MODE => {
            (current & !COLOR_BITS) | color::CM_P256 | (values[0] as u32 & color::PCOLOR_MASK)
        }
        _ => current,
    }
}

fn extract_color(
    attr: &mut AttributeData,
    params: &Params,
    pos: usize,
    target: ColorTarget,
) -> usize {
    // Keep source slot consumption, including its ignored colorspace slot. A
    // malformed last-position subparam can extend the JS array by one slot.
    let mut accu = vec![0, 0, -1, 0, 0, 0];
    let mut color_space = 0;
    let mut advance = 0;
    loop {
        accu[advance + color_space] = params.values()[pos + advance];
        if let Some(sub) = params.get_sub_params(pos + advance) {
            let mut index = 0;
            loop {
                if accu[1] == PALETTE_MODE {
                    color_space = 1;
                }
                let slot = advance + index + 1 + color_space;
                if slot == accu.len() {
                    accu.push(sub[index]);
                } else {
                    accu[slot] = sub[index];
                }
                index += 1;
                if index >= sub.len() || index + advance + 1 + color_space >= accu.len() {
                    break;
                }
            }
            break;
        }
        if (accu[1] == PALETTE_MODE && advance + color_space >= 2)
            || (accu[1] == RGB_MODE && advance + color_space >= 5)
        {
            break;
        }
        if accu[1] != 0 {
            color_space = 1;
        }
        advance += 1;
        if advance + pos >= params.len() || advance + color_space >= accu.len() {
            break;
        }
    }
    for value in &mut accu[2..] {
        if *value == -1 {
            *value = 0;
        }
    }
    let values = [accu[3], accu[4], accu[5]];
    match target {
        ColorTarget::Foreground => attr.fg = update_color(attr.fg, accu[1], values),
        ColorTarget::Background => attr.bg = update_color(attr.bg, accu[1], values),
        ColorTarget::Underline => {
            attr.extended = attr.extended.clone();
            attr.extended.set_underline_color(update_color(
                attr.extended.underline_color(),
                accu[1],
                values,
            ));
        }
    }
    advance
}
