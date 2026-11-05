// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
//! Custom block, path, pattern and braille drawing from addon-webgl/customGlyphs.
mod arc;
#[cfg(target_arch = "wasm32")]
mod browser;
mod definitions;
mod path;
#[cfg(target_arch = "wasm32")]
mod patterns;
mod types;
use self::{
    path::{draw_path, PathMode},
    types::{Area, Definition, Part},
};
#[cfg(target_arch = "wasm32")]
pub use browser::try_draw_custom_glyph;
pub use types::{CustomGlyphDrawParams, DrawCommand, GlyphCanvas};
use DrawCommand as C;

/// Pattern membership is independent of the enabled customGlyphs option.
pub fn is_block_pattern(codepoint: u32) -> bool {
    definitions::is_pattern(codepoint)
}

fn codepoint(chars: &[u16]) -> Option<u32> {
    let mut decoded = char::decode_utf16(chars.iter().copied());
    let value = decoded.next()?.ok()?;
    decoded.next().is_none().then_some(u32::from(value))
}

/// Draw a known complete UTF-16 character, or leave the canvas untouched and
/// return false. The reference's ordinary lookup object also reports inherited
/// Object.prototype names as handled, without drawing a definition part.
pub fn draw_custom_glyph<T: GlyphCanvas>(
    canvas: &mut T,
    chars: &[u16],
    params: CustomGlyphDrawParams<'_>,
) -> Result<bool, T::Error> {
    let Some(parts) = codepoint(chars).and_then(definitions::lookup) else {
        // These standard Object.prototype properties are truthy definitions,
        // but have no definition type. They deliberately draw nothing upstream.
        let inherited = [
            "constructor",
            "__defineGetter__",
            "__defineSetter__",
            "hasOwnProperty",
            "__lookupGetter__",
            "__lookupSetter__",
            "isPrototypeOf",
            "propertyIsEnumerable",
            "toString",
            "valueOf",
            "__proto__",
            "toLocaleString",
        ]
        .iter()
        .any(|name| name.encode_utf16().eq(chars.iter().copied()));
        return Ok(inherited);
    };
    for part in parts {
        draw_part(canvas, *part, params)?;
    }
    Ok(true)
}

fn draw_part<T: GlyphCanvas>(
    canvas: &mut T,
    part: Part,
    p: CustomGlyphDrawParams<'_>,
) -> Result<(), T::Error> {
    let mut area = Area {
        x: p.x_offset,
        y: p.y_offset,
        width: p.device_cell_width,
        height: p.device_cell_height,
    };
    if part.scale_char {
        area.width = p.device_char_width;
        area.height = p.device_char_height;
        area.x += (p.device_cell_width - p.device_char_width) / 2.0;
        area.y += (p.device_cell_height - p.device_char_height) / 2.0;
    }
    if let Some(clip) = part.clip {
        canvas.draw(C::Save)?;
        canvas.draw(C::Begin)?;
        draw_path(canvas, clip, area, PathMode::Clip)?;
        canvas.draw(C::Clip)?;
    }
    match part.data {
        Definition::Blocks(blocks) => {
            for b in blocks {
                let x = area.width / 8.0;
                let y = area.height / 8.0;
                canvas.draw(C::FillRect([
                    area.x + b.x * x,
                    area.y + b.y * y,
                    b.w * x,
                    b.h * y,
                ]))?;
            }
        }
        Definition::Braille(pattern) => {
            let x_unit = area.width / 8.0;
            let padding_y = area.height * 0.1;
            let usable_height = area.height * 0.8;
            let y_unit = usable_height / 8.0;
            let radius = if x_unit.is_nan() || y_unit.is_nan() {
                f64::NAN
            } else {
                x_unit.min(y_unit)
            };
            let positions = [
                [1.0, 0.0],
                [1.0, 2.0],
                [1.0, 4.0],
                [5.0, 0.0],
                [5.0, 2.0],
                [5.0, 4.0],
                [1.0, 6.0],
                [5.0, 6.0],
            ];
            for (bit, [x, y]) in positions.into_iter().enumerate() {
                if pattern & (1 << bit) != 0 {
                    canvas.draw(C::Begin)?;
                    canvas.draw(C::Arc([
                        area.x + (x + 1.0) * x_unit,
                        area.y + padding_y + (y + 1.0) * y_unit,
                        radius,
                        0.0,
                        std::f64::consts::TAU,
                    ]))?;
                    canvas.draw(C::Fill)?;
                }
            }
        }
        Definition::Pattern(id, pattern) => {
            canvas.draw(C::Pattern(id, pattern, p.variant_offset))?;
            canvas.draw(C::FillRect(area.rect()))?;
        }
        Definition::Path(path) => {
            canvas.draw(C::Begin)?;
            draw_path(canvas, path, area, PathMode::Direct)?;
            finish(canvas, part.stroke, p.device_pixel_ratio)?;
        }
        Definition::PathFunction(path) => {
            canvas.draw(C::Save)?;
            canvas.draw(C::Begin)?;
            canvas.draw(C::Rect(area.rect()))?;
            canvas.draw(C::Clip)?;
            canvas.draw(C::Begin)?;
            let instructions = path.instructions(0.15, 0.15 / area.height * area.width);
            draw_path(canvas, &instructions, area, PathMode::Snapped)?;
            finish(canvas, part.stroke, p.device_pixel_ratio)?;
            canvas.draw(C::Close)?;
            canvas.draw(C::Restore)?;
        }
        Definition::Negative(vector) => {
            canvas.draw(C::Save)?;
            canvas.draw(C::FillRect(area.rect()))?;
            if let Some(background) = p.background_color.filter(|s| !s.is_empty()) {
                canvas.draw(C::Background(background.into()))?;
            }
            canvas.draw(C::LineWidth(p.device_pixel_ratio))?;
            canvas.draw(C::SquareCap)?;
            canvas.draw(C::Begin)?;
            draw_path(canvas, vector.path, area, PathMode::Negative)?;
            canvas.draw(if vector.stroke { C::Stroke } else { C::Fill })?;
            canvas.draw(C::Restore)?;
        }
        Definition::Vector(vector) => {
            canvas.draw(C::ClipRect(area.rect()))?;
            canvas.draw(C::Begin)?;
            let css_width = p.font_size / 12.0;
            canvas.draw(C::LineWidth(p.device_pixel_ratio * css_width))?;
            draw_path(
                canvas,
                vector.path,
                area,
                PathMode::Vector {
                    left: vector.left * (css_width / 2.0),
                    right: vector.right * (css_width / 2.0),
                    dpr: p.device_pixel_ratio,
                },
            )?;
            if vector.stroke {
                canvas.draw(C::StrokeFromFill)?;
                canvas.draw(C::Stroke)?;
            } else {
                canvas.draw(C::Fill)?;
            }
            canvas.draw(C::Close)?;
        }
    }
    if part.clip.is_some() {
        canvas.draw(C::Restore)?;
    }
    Ok(())
}

fn finish<T: GlyphCanvas>(canvas: &mut T, width: Option<f64>, dpr: f64) -> Result<(), T::Error> {
    if let Some(width) = width {
        canvas.draw(C::StrokeFromFill)?;
        canvas.draw(C::LineWidth(dpr * width))?;
        canvas.draw(C::Stroke)
    } else {
        canvas.draw(C::Fill)
    }
}
