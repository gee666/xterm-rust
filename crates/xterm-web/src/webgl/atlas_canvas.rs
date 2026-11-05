// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. TextureAtlas.ts post-rasterization canvas processing.
use super::{
    atlas_pixels::{
        clear_background, completely_transparent, find_glyph_bounds, GlyphBounds, GlyphScan,
    },
    gpu_resources::error,
};
use wasm_bindgen::JsValue;
use web_sys::{CanvasRenderingContext2d, ImageData};

#[derive(Clone, Copy, Debug)]
pub struct PixelOptions {
    pub background_rgba: u32,
    pub foreground_rgba: u32,
    pub allow_transparency: bool,
    pub clear_threshold: bool,
}

/// Pixel crop and offsets before atlas placement. The image retains its full
/// original dimensions; packing must copy only bounds, as source putImageData does.
pub struct GlyphBitmap {
    pub image: ImageData,
    pub bounds: GlyphBounds,
}

/// Process an already-rasterized glyph. This implements the filtering/bounds
/// phase, not font drawing, custom glyph rasterization, cache or atlas packing.
pub fn prepare_image(
    image: &ImageData,
    scan: GlyphScan,
    options: PixelOptions,
) -> Result<Option<GlyphBitmap>, JsValue> {
    let mut pixels = image.data().0;
    let empty = if options.allow_transparency {
        completely_transparent(&pixels)
    } else {
        clear_background(
            &mut pixels,
            options.background_rgba,
            options.foreground_rgba,
            options.clear_threshold,
        )
    }
    .map_err(|e| error(&e.to_string()))?;
    if empty {
        return Ok(None);
    }
    let bounds = find_glyph_bounds(
        &pixels,
        image.width() as usize,
        image.height() as usize,
        scan,
    )
    .map_err(|e| error(&e.to_string()))?;
    // ImageData retains its input array. A borrowed WASM slice would outlive the
    // Vec and can be overwritten or detached on memory growth.
    let data = js_sys::Uint8ClampedArray::from(pixels.as_slice());
    let image =
        ImageData::new_with_js_u8_clamped_array_and_sh(&data, image.width(), image.height())?;
    Ok(Some(GlyphBitmap { image, bounds }))
}

/// Read actual canvas pixels with the caller's context. Canvas/security/read
/// failures propagate unchanged; no fallback pixels or font implementation is used.
pub fn prepare_canvas(
    context: &CanvasRenderingContext2d,
    scan: GlyphScan,
    options: PixelOptions,
) -> Result<Option<GlyphBitmap>, JsValue> {
    let canvas = context
        .canvas()
        .ok_or_else(|| error("Glyph context has no HTML canvas"))?;
    let image = context.get_image_data(0.0, 0.0, canvas.width() as f64, canvas.height() as f64)?;
    prepare_image(&image, scan, options)
}
