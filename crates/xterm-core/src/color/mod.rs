// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Ported from src/common/Color.ts.

//! RGBA colors use big-endian channel order, independent of machine endianness.
//! CSS strings are retained where upstream retains them, including letter case.

pub mod css;

/// Upstream `IColor`. `rgba` is packed as `0xrrggbbaa`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Color {
    pub css: String,
    pub rgba: u32,
}

impl Color {
    pub fn null() -> Self {
        Self {
            css: "#00000000".into(),
            rgba: 0,
        }
    }
}

/// Upstream pads to at least two digits, without truncating larger values.
pub fn to_padded_hex(value: i64) -> String {
    let value = if value < 0 {
        format!("-{:x}", value.unsigned_abs())
    } else {
        format!("{value:x}")
    };
    if value.len() < 2 {
        format!("0{value}")
    } else {
        value
    }
}

pub mod channels {
    use super::{to_padded_hex, Color};

    /// Channels are integers, not bytes: upstream's CSS parser accepts values above 255.
    pub fn to_css(r: i64, g: i64, b: i64, a: Option<i64>) -> String {
        let mut result = format!(
            "#{}{}{}",
            to_padded_hex(r),
            to_padded_hex(g),
            to_padded_hex(b)
        );
        if let Some(a) = a {
            result.push_str(&to_padded_hex(a));
        }
        result
    }

    pub fn to_rgba(r: i64, g: i64, b: i64, a: Option<i64>) -> u32 {
        ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | a.unwrap_or(255) as u32
    }

    pub fn to_color(r: i64, g: i64, b: i64, a: Option<i64>) -> Color {
        Color {
            css: to_css(r, g, b, a),
            rgba: to_rgba(r, g, b, a),
        }
    }
}

fn js_round(value: f64) -> f64 {
    let lower = value.floor();
    if value - lower < 0.5 {
        lower
    } else {
        value.ceil()
    }
}

// Opacity is not clamped upstream. Keep the hexadecimal representation of the
// entire rounded JS number, even when its magnitude exceeds Rust integer types.
fn rounded_number_hex(value: f64) -> String {
    if value.is_nan() {
        return "NaN".into();
    }
    if value == f64::INFINITY {
        return "Infinity".into();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".into();
    }
    if value == 0.0 {
        return "00".into();
    }
    let bits = value.abs().to_bits();
    let exponent = ((bits >> 52) & 2047) as i32 - 1023;
    let mantissa = (bits & ((1_u64 << 52) - 1)) | (1_u64 << 52);
    let shift = exponent - 52;
    let digits = if shift < 0 {
        format!("{:x}", mantissa >> -shift)
    } else {
        let mut digits = format!("{:x}", mantissa << (shift % 4));
        digits.push_str(&"0".repeat((shift / 4) as usize));
        digits
    };
    let result = if value < 0.0 {
        format!("-{digits}")
    } else {
        digits
    };
    if result.len() < 2 {
        format!("0{result}")
    } else {
        result
    }
}

/// WCAG contrast ratio, accepting either luminance order.
pub fn contrast_ratio(l1: f64, l2: f64) -> f64 {
    if l1 < l2 {
        (l2 + 0.05) / (l1 + 0.05)
    } else {
        (l1 + 0.05) / (l2 + 0.05)
    }
}

pub mod rgb {
    pub fn relative_luminance(rgb: u32) -> f64 {
        relative_luminance_channels(
            ((rgb >> 16) & 255) as u8,
            ((rgb >> 8) & 255) as u8,
            (rgb & 255) as u8,
        )
    }

    pub fn relative_luminance_channels(r: u8, g: u8, b: u8) -> f64 {
        fn linear(value: u8) -> f64 {
            let value = f64::from(value) / 255.0;
            if value <= 0.03928 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        }
        linear(r) * 0.2126 + linear(g) * 0.7152 + linear(b) * 0.0722
    }
}

pub mod rgba {
    use super::{channels, contrast_ratio, rgb};

    pub fn to_channels(value: u32) -> [u8; 4] {
        value.to_be_bytes()
    }

    pub fn blend(bg: u32, fg: u32) -> u32 {
        if fg & 255 == 255 {
            return fg;
        }
        let alpha = f64::from(fg & 255) / 255.0;
        let background = to_channels(bg);
        let foreground = to_channels(fg);
        let mut result = [0_i64; 3];
        for i in 0..3 {
            // JS Math.round ties towards +infinity, including negative deltas.
            result[i] = i64::from(background[i])
                + (f64::from(i32::from(foreground[i]) - i32::from(background[i])) * alpha + 0.5)
                    .floor() as i64;
        }
        channels::to_rgba(result[0], result[1], result[2], None)
    }

    pub fn ensure_contrast_ratio(bg: u32, fg: u32, ratio: f64) -> Option<u32> {
        let bg_l = rgb::relative_luminance(bg >> 8);
        let fg_l = rgb::relative_luminance(fg >> 8);
        if contrast_ratio(bg_l, fg_l) >= ratio || ratio.is_nan() {
            return None;
        }
        let first = if fg_l < bg_l {
            reduce_luminance(bg, fg, ratio)
        } else {
            increase_luminance(bg, fg, ratio)
        };
        let first_ratio = contrast_ratio(bg_l, rgb::relative_luminance(first >> 8));
        if first_ratio >= ratio {
            return Some(first);
        }
        let second = if fg_l < bg_l {
            increase_luminance(bg, fg, ratio)
        } else {
            reduce_luminance(bg, fg, ratio)
        };
        let second_ratio = contrast_ratio(bg_l, rgb::relative_luminance(second >> 8));
        Some(if first_ratio > second_ratio {
            first
        } else {
            second
        })
    }

    fn adjust_luminance(bg: u32, fg: u32, ratio: f64, increase: bool) -> u32 {
        let bg_l = rgb::relative_luminance(bg >> 8);
        let [mut r, mut g, mut b, _] = to_channels(fg);
        while contrast_ratio(bg_l, rgb::relative_luminance_channels(r, g, b)) < ratio {
            if (increase && r == 255 && g == 255 && b == 255)
                || (!increase && r == 0 && g == 0 && b == 0)
            {
                break;
            }
            for channel in [&mut r, &mut g, &mut b] {
                // The source moves ten percent towards the endpoint, rounding up.
                let distance = if increase { 255 - *channel } else { *channel };
                let step = (f64::from(distance) * 0.1).ceil() as u8;
                if increase {
                    *channel += step;
                } else {
                    *channel -= step;
                }
            }
        }
        u32::from_be_bytes([r, g, b, 255])
    }

    pub fn reduce_luminance(bg: u32, fg: u32, ratio: f64) -> u32 {
        adjust_luminance(bg, fg, ratio, false)
    }
    pub fn increase_luminance(bg: u32, fg: u32, ratio: f64) -> u32 {
        adjust_luminance(bg, fg, ratio, true)
    }
}

/// Operations on `Color`, corresponding to upstream's `color` namespace.
pub mod value {
    use super::{channels, rgba, Color};

    pub fn blend(bg: &Color, fg: &Color) -> Color {
        if is_opaque(fg) {
            return fg.clone();
        }
        let [r, g, b, _] = rgba::to_channels(rgba::blend(bg.rgba, fg.rgba));
        channels::to_color(r.into(), g.into(), b.into(), None)
    }
    pub fn is_opaque(color: &Color) -> bool {
        color.rgba & 255 == 255
    }
    pub fn ensure_contrast_ratio(bg: &Color, fg: &Color, ratio: f64) -> Option<Color> {
        let [r, g, b, _] = rgba::to_channels(rgba::ensure_contrast_ratio(bg.rgba, fg.rgba, ratio)?);
        Some(channels::to_color(r.into(), g.into(), b.into(), None))
    }
    pub fn opaque(color: &Color) -> Color {
        let [r, g, b, _] = rgba::to_channels(color.rgba);
        channels::to_color(r.into(), g.into(), b.into(), None)
    }
    /// Opacity follows JavaScript rounding and wrapping, including out-of-range factors.
    pub fn opacity(color: &Color, opacity: f64) -> Color {
        let alpha = super::js_round(opacity * 255.0);
        let [r, g, b, _] = rgba::to_channels(color.rgba);
        let alpha_integer = if alpha.is_finite() {
            alpha.rem_euclid(4294967296.0) as u32
        } else {
            0
        };
        let alpha_css = super::rounded_number_hex(alpha);
        Color {
            css: format!(
                "{}{}",
                channels::to_css(r.into(), g.into(), b.into(), None),
                alpha_css
            ),
            rgba: (color.rgba & 0xffffff00) | alpha_integer,
        }
    }
    pub fn multiply_opacity(color: &Color, factor: f64) -> Color {
        opacity(color, (f64::from(color.rgba & 255) * factor) / 255.0)
    }
    pub fn to_color_rgb(color: &Color) -> [u8; 3] {
        let [r, g, b, _] = rgba::to_channels(color.rgba);
        [r, g, b]
    }
}
