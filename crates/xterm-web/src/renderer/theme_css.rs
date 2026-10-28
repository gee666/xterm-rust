// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
//! Exact theme text from DomRenderer._injectCss, without owning DOM nodes.
#[path = "theme_css_rules.rs"]
pub(crate) mod rules;
use super::width_cache::Font;
use crate::theme::{ColorRef, ColorSet};
use rules::{ColorField, CssReader, Field, OptionField};
use std::convert::Infallible;
use xterm_core::color::value::{multiply_opacity, opaque};

/// Well-formed native-string convenience path. Live browser options use the
/// UTF-16 reader so coercion callbacks occur at their original source positions.
pub fn theme_css(id: u64, colors: &ColorSet, font: &Font, cursor_width: f64) -> String {
    // All native inputs are Rust strings, finite or non-finite ASCII number
    // spellings, or the fixed Rust template. Their UTF-16 is always well formed.
    String::from_utf16(&snapshot_units(id, colors, font, cursor_width, false))
        .expect("native CSS inputs and template are well-formed Unicode")
}

pub(super) fn snapshot_units(
    id: u64,
    colors: &ColorSet,
    font: &Font,
    cursor_width: f64,
    raw_colors: bool,
) -> Vec<u16> {
    let source = Native {
        colors,
        font,
        cursor_width,
        raw_colors,
    };
    rules::build(id, &source).unwrap_or_else(|never| match never {})
}
struct Native<'a> {
    colors: &'a ColorSet,
    font: &'a Font,
    cursor_width: f64,
    raw_colors: bool,
}
impl CssReader for Native<'_> {
    type Error = Infallible;
    fn palette_len(&self) -> usize {
        self.colors.ansi.len()
    }
    fn read(&self, field: Field) -> Result<Vec<u16>, Infallible> {
        Ok(match field {
            Field::Color(field) => color_units(self.colors, field, self.raw_colors),
            Field::Option(field) => {
                let value = match field {
                    OptionField::Family => self.font.family.clone(),
                    OptionField::Size => number(self.font.size),
                    OptionField::Weight => self.font.weight.clone(),
                    OptionField::WeightBold => self.font.weight_bold.clone(),
                    OptionField::CursorWidth => number(self.cursor_width),
                };
                value.encode_utf16().collect()
            }
        })
    }
}

pub(super) fn color_units(colors: &ColorSet, field: ColorField, raw: bool) -> Vec<u16> {
    use ColorField::*;
    let computed = |color: &ColorRef, opaque_first: bool, half: bool| {
        let mut value = if opaque_first {
            opaque(&color.borrow())
        } else {
            color.borrow().clone()
        };
        if half {
            value = multiply_opacity(&value, 0.5);
        }
        value.css.encode_utf16().collect()
    };
    // The generator iterates the current dense palette only after the final
    // option-coercion callback. No external callback occurs inside that loop.
    let direct = match field {
        Foreground => &colors.foreground,
        ForegroundHalf => return computed(&colors.foreground, false, true),
        BackgroundOpaque => return computed(&colors.background, true, false),
        BackgroundOpaqueHalf => return computed(&colors.background, true, true),
        Cursor => &colors.cursor,
        CursorAccent => &colors.cursor_accent,
        Selection => &colors.selection_background_opaque,
        InactiveSelection => &colors.selection_inactive_background_opaque,
        Ansi(index) => &colors.ansi[index],
        AnsiHalf(index) => return computed(&colors.ansi[index], false, true),
    };
    if raw {
        direct.css_units()
    } else {
        direct.borrow().css.encode_utf16().collect()
    }
}

fn number(value: f64) -> String {
    ryu_js::Buffer::new().format(value).to_owned()
}
