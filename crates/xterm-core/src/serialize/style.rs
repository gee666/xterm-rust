// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use crate::buffer::AttributeData;

pub(super) fn equal_bg(a: &AttributeData, b: &AttributeData) -> bool {
    a.get_bg_color_mode() == b.get_bg_color_mode() && a.get_bg_color() == b.get_bg_color()
}

pub(super) fn equal_underline(a: &AttributeData, b: &AttributeData) -> bool {
    if a.is_underline() == 0 && b.is_underline() == 0 {
        return true;
    }
    if a.get_underline_style() != b.get_underline_style() {
        return false;
    }
    let (ad, bd) = (
        a.is_underline_color_default(),
        b.is_underline_color_default(),
    );
    (ad && bd)
        || (ad == bd
            && a.get_underline_color() == b.get_underline_color()
            && a.get_underline_color_mode() == b.get_underline_color_mode())
}

pub(super) fn equal_flags(a: &AttributeData, b: &AttributeData) -> bool {
    a.is_inverse() == b.is_inverse()
        && a.is_bold() == b.is_bold()
        && a.is_underline() == b.is_underline()
        && equal_underline(a, b)
        && a.is_overline() == b.is_overline()
        && a.is_blink() == b.is_blink()
        && a.is_invisible() == b.is_invisible()
        && a.is_italic() == b.is_italic()
        && a.is_dim() == b.is_dim()
        && a.is_strikethrough() == b.is_strikethrough()
}

pub(super) fn equal_fg(a: &AttributeData, b: &AttributeData) -> bool {
    a.get_fg_color_mode() == b.get_fg_color_mode() && a.get_fg_color() == b.get_fg_color()
}

pub(super) fn diff(a: &AttributeData, b: &AttributeData) -> String {
    let (fg, bg, flags) = (!equal_fg(a, b), !equal_bg(a, b), !equal_flags(a, b));
    if !(fg || bg || flags) {
        return String::new();
    }
    if a.is_attribute_default() {
        return if b.is_attribute_default() {
            String::new()
        } else {
            "0".into()
        };
    }
    let mut codes = Vec::new();
    if fg {
        color(
            &mut codes,
            a.get_fg_color(),
            a.is_fg_rgb(),
            a.is_fg_palette(),
            true,
        );
    }
    if bg {
        color(
            &mut codes,
            a.get_bg_color(),
            a.is_bg_rgb(),
            a.is_bg_palette(),
            false,
        );
    }
    if flags {
        flag(&mut codes, a.is_inverse(), b.is_inverse(), 7, 27);
        flag(&mut codes, a.is_bold(), b.is_bold(), 1, 22);
        if !equal_underline(a, b) {
            let style = a.get_underline_style();
            if style == 0 {
                codes.push("24".into());
            } else if style == 1 && a.is_underline_color_default() {
                codes.push("4".into());
            } else {
                codes.push(format!("4:{style}"));
                if !a.is_underline_color_default() {
                    let c = a.get_underline_color();
                    codes.push(if a.is_underline_color_rgb() {
                        format!("58:2::{}:{}:{}", (c >> 16) & 255, (c >> 8) & 255, c & 255)
                    } else {
                        format!("58:5:{c}")
                    });
                }
            }
        } else {
            flag(&mut codes, a.is_underline(), b.is_underline(), 4, 24);
        }
        flag(&mut codes, a.is_overline(), b.is_overline(), 53, 55);
        flag(&mut codes, a.is_blink(), b.is_blink(), 5, 25);
        flag(&mut codes, a.is_invisible(), b.is_invisible(), 8, 28);
        flag(&mut codes, a.is_italic(), b.is_italic(), 3, 23);
        flag(&mut codes, a.is_dim(), b.is_dim(), 2, 22);
        flag(
            &mut codes,
            a.is_strikethrough(),
            b.is_strikethrough(),
            9,
            29,
        );
    }
    codes.join(";")
}

fn flag(out: &mut Vec<String>, a: u32, b: u32, on: u8, off: u8) {
    if a != b {
        out.push(if a != 0 { on } else { off }.to_string());
    }
}

fn color(out: &mut Vec<String>, c: i32, rgb: bool, palette: bool, fg: bool) {
    let base = if fg { 30 } else { 40 };
    if rgb {
        out.extend([base + 8, 2, (c >> 16) & 255, (c >> 8) & 255, c & 255].map(|n| n.to_string()));
    } else if palette {
        if c >= 16 {
            out.extend([base + 8, 5, c].map(|n| n.to_string()));
        } else {
            out.push((base + (if c & 8 != 0 { 60 } else { 0 }) + (c & 7)).to_string());
        }
    } else {
        out.push((base + 9).to_string());
    }
}
