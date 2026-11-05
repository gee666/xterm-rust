// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
//! Pattern identity is cached by immutable definition identity and current color.
use js_sys::{Reflect, Uint8ClampedArray};
use std::{cell::RefCell, collections::HashMap};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{
    CanvasPattern, CanvasRenderingContext2d, HtmlCanvasElement, ImageData, OffscreenCanvas,
    OffscreenCanvasRenderingContext2d,
};

// web-sys 0.3.77 types this API as the obsolete SVGMatrix. Use a narrow Web API
// binding rather than fabricating that type or moving rendering to JavaScript.
#[wasm_bindgen(
    inline_js = "export function customGlyphPatternTransform(pattern, x, y) { pattern.setTransform(new DOMMatrix().translateSelf(x,y)); }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = customGlyphPatternTransform)]
    fn transform(pattern: &CanvasPattern, x: f64, y: f64) -> Result<(), JsValue>;
}

type PatternKey = (u16, String);
thread_local! { static PATTERNS: RefCell<HashMap<PatternKey, CanvasPattern>> = RefCell::default(); }
fn error(message: &str) -> JsValue {
    js_sys::Error::new(message).into()
}

fn color(style: &str) -> Result<[f64; 4], JsValue> {
    if style.starts_with('#') {
        let component = |start: usize| {
            style
                .get(start..start + 2)
                .and_then(|s| u8::from_str_radix(s, 16).ok())
                .map(f64::from)
                .unwrap_or(f64::NAN)
        };
        let alpha = component(7);
        // Source's `parseInt(...) || 1` deliberately makes #rrggbb00 opaque.
        Ok([
            component(1),
            component(3),
            component(5),
            if alpha.is_nan() || alpha == 0.0 {
                1.0
            } else {
                alpha
            },
        ])
    } else if style.starts_with("rgba") {
        let mut rgba = [f64::NAN; 4];
        if let Some(body) = style.get(5..style.len().saturating_sub(1)) {
            for (target, part) in rgba.iter_mut().zip(body.split(',')) {
                *target = part.trim().parse().unwrap_or(f64::NAN);
            }
        }
        Ok(rgba)
    } else {
        Err(error(&format!(
            "Unexpected fillStyle color format \"{style}\" when drawing pattern glyph"
        )))
    }
}

fn create_pattern(
    ctx: &CanvasRenderingContext2d,
    rows: &[&[u8]],
    style: &str,
) -> Result<CanvasPattern, JsValue> {
    // All patterns are validated, tiny generated definition data.
    let width = rows[0].len() as u32;
    let height = rows.len() as u32;
    let rgba = color(style)?;
    let mut pixels = Vec::with_capacity(width as usize * height as usize * 4);
    for row in rows {
        for &value in *row {
            pixels.extend_from_slice(&[
                rgba[0],
                rgba[1],
                rgba[2],
                f64::from(value) * (rgba[3] * 255.0),
            ]);
        }
    }
    // Conversion must use ECMAScript Uint8ClampedArray's ties-to-even semantics.
    let array = Uint8ClampedArray::new_with_length(pixels.len() as u32);
    for (index, value) in pixels.into_iter().enumerate() {
        Reflect::set(array.as_ref(), &(index as u32).into(), &value.into())?;
    }
    let image = ImageData::new_with_js_u8_clamped_array_and_sh(&array, width, height)?;
    let pattern = if Reflect::get(&js_sys::global(), &"OffscreenCanvas".into())?.is_undefined() {
        // Use the main realm document, not the atlas canvas's mutable ownerDocument.
        let document = web_sys::window()
            .and_then(|w| w.document())
            .ok_or_else(|| error("Pattern canvas document unavailable"))?;
        let canvas: HtmlCanvasElement = document.create_element("canvas")?.dyn_into()?;
        canvas.set_width(width);
        canvas.set_height(height);
        let context: CanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| error("Value must not be falsy"))?
            .dyn_into()?;
        context.put_image_data(&image, 0.0, 0.0)?;
        ctx.create_pattern_with_html_canvas_element(&canvas, "repeat")?
    } else {
        let canvas = OffscreenCanvas::new(width, height)?;
        let context: OffscreenCanvasRenderingContext2d = canvas
            .get_context("2d")?
            .ok_or_else(|| error("Value must not be falsy"))?
            .dyn_into()?;
        context.put_image_data(&image, 0.0, 0.0)?;
        ctx.create_pattern_with_offscreen_canvas(&canvas, "repeat")?
    };
    pattern.ok_or_else(|| error("Value must not be falsy"))
}

pub(super) fn set_pattern(
    ctx: &CanvasRenderingContext2d,
    id: u16,
    rows: &'static [&'static [u8]],
    variant: i32,
) -> Result<(), JsValue> {
    let fill = ctx.fill_style();
    let style = fill.as_string().ok_or_else(|| {
        // Native gradients/patterns stringify through the same template semantics.
        let text = xterm_wasm_support::options::from_js(&fill).and_then(|value| {
            value
                .to_utf16()
                .map_err(|error| xterm_wasm_support::js_error::to_js_error(&error))
        });
        match text {
            Ok(text) => error(&format!(
                "Unexpected fillStyle type \"{}\"",
                String::from_utf16_lossy(&text)
            )),
            Err(error) => error,
        }
    })?;
    let key = (id, style.clone());
    let cached = PATTERNS.with(|cache| cache.borrow().get(&key).cloned());
    let pattern = match cached {
        Some(pattern) => pattern,
        None => {
            let pattern = create_pattern(ctx, rows, &style)?;
            PATTERNS.with(|cache| cache.borrow_mut().insert(key, pattern.clone()));
            pattern
        }
    };
    transform(
        &pattern,
        -f64::from((variant >> 1) & 1),
        -f64::from(variant & 1),
    )?;
    ctx.set_fill_style_canvas_pattern(&pattern);
    Ok(())
}
