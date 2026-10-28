// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Port of browser/renderer/shared/RendererUtils.ts.

use crate::api::RenderDimensions;
use std::{error::Error, fmt};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FalsyValue;
impl fmt::Display for FalsyValue {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("value must not be falsy")
    }
}
impl Error for FalsyValue {}

/// JavaScript truthiness belongs at the actual JS boundary; native typed
/// object handles instead use Option::ok_or(FalsyValue) for missing objects.
#[cfg(target_arch = "wasm32")]
pub fn throw_if_falsy(value: wasm_bindgen::JsValue) -> Result<wasm_bindgen::JsValue, FalsyValue> {
    if value.is_falsy() {
        Err(FalsyValue)
    } else {
        Ok(value)
    }
}

pub fn is_powerline_glyph(codepoint: u32) -> bool {
    (0xe0a4..=0xe0d6).contains(&codepoint)
}
pub fn is_restricted_powerline_glyph(codepoint: u32) -> bool {
    (0xe0b0..=0xe0b7).contains(&codepoint)
}
fn is_nerd_font_glyph(codepoint: u32) -> bool {
    (0xe000..=0xf8ff).contains(&codepoint)
}
fn is_box_or_block_glyph(codepoint: u32) -> bool {
    (0x2500..=0x259f).contains(&codepoint)
}

pub fn is_emoji(codepoint: u32) -> bool {
    (0x1f600..=0x1f64f).contains(&codepoint)
        || (0x1f300..=0x1f5ff).contains(&codepoint)
        || (0x1f680..=0x1f6ff).contains(&codepoint)
        || (0x2600..=0x26ff).contains(&codepoint)
        || (0x2700..=0x27bf).contains(&codepoint)
        || (0xfe00..=0xfe0f).contains(&codepoint)
        || (0x1f900..=0x1f9ff).contains(&codepoint)
        || (0x1f1e6..=0x1f1ff).contains(&codepoint)
}

pub fn allow_rescaling(
    codepoint: Option<i64>,
    width: f64,
    glyph_size_x: f64,
    device_cell_width: f64,
) -> bool {
    width == 1.0
        && glyph_size_x > (device_cell_width * 1.5).ceil()
        && codepoint.is_some_and(|codepoint| {
            // Combined-cell marker writes produce negative JS int32 values.
            // Do not reinterpret those as large positive Unicode codepoints.
            codepoint > 0xff
                && match u32::try_from(codepoint) {
                    Ok(codepoint) => {
                        !is_emoji(codepoint)
                            && !is_powerline_glyph(codepoint)
                            && !is_nerd_font_glyph(codepoint)
                    }
                    // Every excluded range is within u32; larger positive numeric
                    // values therefore remain outside those source ranges.
                    Err(_) => true,
                }
        })
}

pub fn treat_glyph_as_background_color(codepoint: u32) -> bool {
    is_powerline_glyph(codepoint) || is_box_or_block_glyph(codepoint)
}

pub fn create_render_dimensions() -> RenderDimensions {
    RenderDimensions::default()
}

pub fn compute_next_variant_offset(
    cell_width: f64,
    line_width: f64,
    current_offset: Option<f64>,
) -> f64 {
    let step = js_round(line_width) * 2.0;
    (cell_width - (step - current_offset.unwrap_or(0.0))) % step
}

fn js_round(value: f64) -> f64 {
    if !value.is_finite() || value == 0.0 {
        return value;
    }
    // JS ties toward +infinity and retains negative zero for [-0.5, 0).
    if (-0.5..0.0).contains(&value) {
        return -0.0;
    }
    let lower = value.floor();
    if value - lower < 0.5 {
        lower
    } else {
        lower + 1.0
    }
}
