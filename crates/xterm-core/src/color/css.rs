// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from src/common/Color.ts.

//! Fast CSS formats shared by headless and browser terminals. Browser callers
//! supply canvas parsing for other CSS formats, preserving upstream validation.

use super::{channels, Color};
use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UnsupportedCss;

impl fmt::Display for UnsupportedCss {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("css.toColor: Unsupported css format")
    }
}
impl Error for UnsupportedCss {}

pub fn to_color(css: &str) -> Result<Color, UnsupportedCss> {
    to_color_with_canvas(css, |_| None)
}

/// `canvas` must return the canvas pixel after validating fillStyle assignment.
/// Only opaque fallback colors are accepted, as the source rejects colors whose
/// hue could have been lost by canvas alpha premultiplication.
pub fn to_color_with_canvas(
    css: &str,
    canvas: impl FnOnce(&str) -> Option<[u8; 4]>,
) -> Result<Color, UnsupportedCss> {
    let units: Vec<u16> = css.encode_utf16().collect();
    // Upstream's hexadecimal regular expression is intentionally unanchored.
    let has_hex = units
        .windows(4)
        .any(|s| s[0] == u16::from(b'#') && s[1..].iter().all(|&c| hex(c).is_some()));
    if has_hex {
        let parse = |start, end| parse_hex_prefix(&units[start..end]);
        match units.len() {
            4 | 5 => {
                let r = parse(1, 2) * 17;
                let g = parse(2, 3) * 17;
                let b = parse(3, 4) * 17;
                let a = if units.len() == 5 {
                    Some(i64::from(parse(4, 5) * 17))
                } else {
                    None
                };
                let mut result = channels::to_color(r.into(), g.into(), b.into(), a);
                // An unanchored match can leave a non-hex channel at the start.
                // JS renders parseInt's NaN as "NaN", but bitwise packing uses zero.
                if units[1..].iter().any(|&c| hex(c).is_none()) {
                    result.css = "#".into();
                    for &unit in &units[1..] {
                        if let Some(digit) = hex(unit) {
                            result
                                .css
                                .push_str(&super::to_padded_hex(i64::from(digit * 17)));
                        } else {
                            result.css.push_str("NaN");
                        }
                    }
                }
                return Ok(result);
            }
            7 => {
                return Ok(Color {
                    css: css.into(),
                    rgba: (parse(1, 7) << 8) | 255,
                })
            }
            9 => {
                return Ok(Color {
                    css: css.into(),
                    rgba: parse(1, 9),
                })
            }
            _ => {}
        }
    }
    if let Some([r, g, b, a]) = parse_rgb(css) {
        return Ok(channels::to_color(r, g, b, Some(a)));
    }
    if css == "transparent" {
        return Ok(Color {
            css: css.into(),
            rgba: 0,
        });
    }
    match canvas(css) {
        Some([r, g, b, 255]) => Ok(Color {
            css: css.into(),
            rgba: u32::from_be_bytes([r, g, b, 255]),
        }),
        _ => Err(UnsupportedCss),
    }
}

fn hex(c: u16) -> Option<u32> {
    match c {
        48..=57 => Some(u32::from(c - 48)),
        65..=70 => Some(u32::from(c - 65 + 10)),
        97..=102 => Some(u32::from(c - 97 + 10)),
        _ => None,
    }
}

fn parse_hex_prefix(units: &[u16]) -> u32 {
    let mut result = 0_u32;
    for &c in units {
        let Some(digit) = hex(c) else {
            break;
        };
        result = result.wrapping_mul(16).wrapping_add(digit);
    }
    result
}

fn js_whitespace(c: char) -> bool {
    matches!(c, '\u{0009}'..='\u{000d}' | ' ' | '\u{00a0}' | '\u{1680}' | '\u{2000}'..='\u{200a}' | '\u{2028}' | '\u{2029}' | '\u{202f}' | '\u{205f}' | '\u{3000}' | '\u{feff}')
}

struct RgbParser<'a> {
    rest: &'a str,
}
impl RgbParser<'_> {
    fn whitespace(&mut self) {
        self.rest = self.rest.trim_start_matches(js_whitespace);
    }
    fn consume(&mut self, c: char) -> bool {
        if let Some(rest) = self.rest.strip_prefix(c) {
            self.rest = rest;
            true
        } else {
            false
        }
    }
    fn channel(&mut self) -> Option<i64> {
        self.whitespace();
        let end = self
            .rest
            .bytes()
            .take(3)
            .take_while(u8::is_ascii_digit)
            .count();
        if end == 0 {
            return None;
        }
        let channel = self.rest[..end].parse().ok()?;
        self.rest = &self.rest[end..];
        self.whitespace();
        Some(channel)
    }
    fn alpha(&mut self) -> Option<i64> {
        self.whitespace();
        let bytes = self.rest.as_bytes();
        let decimal_start = usize::from(bytes.first().is_some_and(u8::is_ascii_digit));
        let end = if bytes.get(decimal_start) == Some(&b'.') {
            let end = decimal_start
                + 1
                + bytes[decimal_start + 1..]
                    .iter()
                    .take_while(|b| b.is_ascii_digit())
                    .count();
            if end == decimal_start + 1 {
                return None;
            }
            end
        } else if matches!(bytes.first(), Some(b'0' | b'1')) {
            1
        } else {
            return None;
        };
        let alpha: f64 = self.rest[..end].parse().ok()?;
        self.rest = &self.rest[end..];
        self.whitespace();
        Some((alpha * 255.0 + 0.5).floor() as i64)
    }
    fn parse(&mut self) -> Option<[i64; 4]> {
        let r = self.channel()?;
        if !self.consume(',') {
            return None;
        }
        let g = self.channel()?;
        if !self.consume(',') {
            return None;
        }
        let b = self.channel()?;
        let a = if self.consume(',') {
            self.alpha()?
        } else {
            255
        };
        if !self.consume(')') {
            return None;
        }
        Some([r, g, b, a])
    }
}

fn parse_rgb(css: &str) -> Option<[i64; 4]> {
    // As in JS, both names permit either three or four components, and a match
    // can occur inside an otherwise invalid string.
    for (index, _) in css.match_indices("rgb") {
        let suffix = &css[index + 3..];
        let suffix = suffix.strip_prefix('a').unwrap_or(suffix);
        if let Some(rest) = suffix.strip_prefix('(') {
            if let Some(result) = (RgbParser { rest }).parse() {
                return Some(result);
            }
        }
    }
    None
}
