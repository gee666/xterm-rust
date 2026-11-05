// Copyright (c) 2021 The xterm.js authors. All rights reserved. MIT license.
//! Pattern identity is cached by immutable definition identity and current color.
use std::{cell::RefCell, collections::HashMap};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{CanvasPattern, CanvasRenderingContext2d};

// The typed web-sys overloads cannot preserve null repetition, ImageData argument
// shape, dynamic data access or modern DOMMatrix calls. Keep only those Web API
// operations in narrow catching bindings; pixel generation remains Rust.
#[wasm_bindgen(module = "/src/webgl/custom_glyphs/patterns.js")]
extern "C" {
    #[wasm_bindgen(js_name = customGlyphHasOffscreenFactory)]
    fn has_offscreen_factory() -> bool;
    #[wasm_bindgen(catch, js_name = customGlyphOffscreenCanvas)]
    fn offscreen_canvas(width: u32, height: u32) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = customGlyphDomCanvas)]
    fn dom_canvas(width: u32, height: u32) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = customGlyphPatternContext)]
    fn context(canvas: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = customGlyphPatternImage)]
    fn image(width: u32, height: u32) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = customGlyphPatternByte)]
    fn set_byte(image: &JsValue, index: u32, value: f64) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = customGlyphPatternPut)]
    fn put(context: &JsValue, image: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = customGlyphPatternCreate)]
    fn create(ctx: &CanvasRenderingContext2d, canvas: &JsValue) -> Result<JsValue, JsValue>;
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
    // These dimensions come only from immutable generated pattern definitions.
    let width = u32::try_from(rows[0].len()).map_err(|_| error("Pattern width exceeds u32"))?;
    let height = u32::try_from(rows.len()).map_err(|_| error("Pattern height exceeds u32"))?;
    let canvas = if has_offscreen_factory() {
        offscreen_canvas(width, height)?
    } else {
        dom_canvas(width, height)?
    };
    let context = context(&canvas)?;
    if !context.is_truthy() {
        return Err(error("Value must not be falsy"));
    }
    // Source creates both host resources before inspecting the color format.
    let image = image(width, height)?;
    let rgba = color(style)?;
    let mut index = 0u32;
    for row in rows {
        for &value in *row {
            for component in [
                rgba[0],
                rgba[1],
                rgba[2],
                f64::from(value) * (rgba[3] * 255.0),
            ] {
                // Each source assignment rereads image.data. It may throw or
                // return another view, and Uint8ClampedArray performs rounding.
                set_byte(&image, index, component)?;
                index = index
                    .checked_add(1)
                    .ok_or_else(|| error("Pattern offset exceeds u32"))?;
            }
        }
    }
    put(&context, &image)?;
    let pattern = create(ctx, &canvas)?;
    if !pattern.is_truthy() {
        return Err(error("Value must not be falsy"));
    }
    Ok(pattern.unchecked_into())
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
