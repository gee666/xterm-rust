// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. TextureAtlas.ts post-rasterization canvas processing.
use super::{
    atlas_pixels::{
        clear_background, completely_transparent, find_glyph_bounds, GlyphBounds, GlyphScan,
    },
    gpu_resources::error,
};
use wasm_bindgen::{JsCast, JsValue};
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

fn processed_pixels(
    image: &ImageData,
    scan: GlyphScan,
    options: PixelOptions,
) -> Result<(Vec<u8>, Option<GlyphBounds>), JsValue> {
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
    let bounds = if empty {
        None
    } else {
        Some(
            find_glyph_bounds(
                &pixels,
                image.width() as usize,
                image.height() as usize,
                scan,
            )
            .map_err(|e| error(&e.to_string()))?,
        )
    };
    Ok((pixels, bounds))
}

/// Copy-producing component API. The caller's ImageData remains unchanged and
/// the returned image owns its JS array independently of WASM memory growth.
pub fn prepare_image(
    image: &ImageData,
    scan: GlyphScan,
    options: PixelOptions,
) -> Result<Option<GlyphBitmap>, JsValue> {
    let (pixels, bounds) = processed_pixels(image, scan, options)?;
    let Some(bounds) = bounds else {
        return Ok(None);
    };
    let data = js_sys::Uint8ClampedArray::from(pixels.as_slice());
    let image =
        ImageData::new_with_js_u8_clamped_array_and_sh(&data, image.width(), image.height())?;
    Ok(Some(GlyphBitmap { image, bounds }))
}

/// The live atlas processes getImageData's existing JS-owned array in place.
/// Creating a replacement ImageData here would add an observable constructor
/// call that the source does not make. Its pixels already outlive a WASM slice.
pub fn prepare_canvas(
    context: &CanvasRenderingContext2d,
    scan: GlyphScan,
    options: PixelOptions,
) -> Result<Option<GlyphBitmap>, JsValue> {
    let canvas = context
        .canvas()
        .ok_or_else(|| error("Glyph context has no HTML canvas"))?;
    let image = context.get_image_data(0.0, 0.0, canvas.width() as f64, canvas.height() as f64)?;
    let (pixels, bounds) = processed_pixels(&image, scan, options)?;
    if !options.allow_transparency {
        let data = js_sys::Reflect::get(image.as_ref(), &"data".into())?
            .dyn_into::<js_sys::Uint8ClampedArray>()
            .map_err(|_| js_sys::TypeError::new("ImageData.data must be a Uint8ClampedArray"))?;
        if data.length() as usize != pixels.len() {
            return Err(
                js_sys::TypeError::new("ImageData.data changed length during processing").into(),
            );
        }
        data.copy_from(&pixels);
    }
    Ok(bounds.map(|bounds| GlyphBitmap { image, bounds }))
}
