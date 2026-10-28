// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{
    js::{call, context, get, nullish, number, set},
    primitives::{create_canvas, create_image_bitmap, create_image_data, int32},
    BrowserImageRenderer,
};
use wasm_bindgen::prelude::*;
use xterm_addons::image::storage::ImageLayer;

const BLUEPRINT_WIDTH: u32 = 32;
const PLACEHOLDER_HEIGHT: f64 = 24.0;
const PLACEHOLDER_LENGTH: i32 = 4096;
#[wasm_bindgen(module = "/src/image/placeholder.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = assignPlaceholderBitmap)]
    fn assign_bitmap(slot: &JsValue, promise: &JsValue) -> Result<(), JsValue>;
}

impl BrowserImageRenderer {
    pub(super) fn clear_placeholder(&self) -> Result<(), JsValue> {
        let bitmap = get(&self.placeholder, "bitmap")?;
        if !nullish(&bitmap) {
            call(&bitmap, "close", &[])?;
        }
        set(&self.placeholder, "bitmap", &JsValue::UNDEFINED)?;
        set(&self.placeholder, "canvas", &JsValue::UNDEFINED)
    }

    /// Refresh notification belongs to the terminal lifecycle owner, which must
    /// call refreshRows after this operation even when metrics are unavailable.
    pub fn show_placeholder(&self, value: bool) -> Result<(), JsValue> {
        if value {
            if nullish(&get(&self.placeholder, "canvas")?) && self.cell_size()?.height != -1.0 {
                self.create_placeholder((self.cell_size()?.height + 1.0).max(PLACEHOLDER_HEIGHT))?;
            }
        } else {
            self.clear_placeholder()?;
        }
        Ok(())
    }

    pub fn draw_placeholder(&self, col: f64, row: f64, count: f64) -> Result<(), JsValue> {
        let Some(ctx) = self.context(ImageLayer::Top) else {
            return Ok(());
        };
        let size = self.cell_size()?;
        if size.width == -1.0 || size.height == -1.0 {
            return Ok(());
        }
        let placeholder = get(&self.placeholder, "canvas")?;
        if nullish(&placeholder) {
            self.create_placeholder((size.height + 1.0).max(PLACEHOLDER_HEIGHT))?;
        } else if size.height >= number(&placeholder, "height")? {
            self.create_placeholder(size.height + 1.0)?;
        }
        let placeholder = get(&self.placeholder, "canvas")?;
        if nullish(&placeholder) {
            return Ok(());
        }
        let bitmap = get(&self.placeholder, "bitmap")?;
        let source = if nullish(&bitmap) {
            placeholder
        } else {
            bitmap
        };
        let dx = col * size.width;
        let dy = row * size.height;
        let sy = if (dy % 2.0) != 0.0 && !(dy % 2.0).is_nan() {
            0.0
        } else {
            1.0
        };
        call(
            &ctx,
            "drawImage",
            &[
                source,
                dx.into(),
                sy.into(),
                (size.width * count).into(),
                size.height.into(),
                dx.into(),
                dy.into(),
                (size.width * count).into(),
                size.height.into(),
            ],
        )?;
        Ok(())
    }

    fn create_placeholder(&self, height: f64) -> Result<(), JsValue> {
        let bitmap = get(&self.placeholder, "bitmap")?;
        if !nullish(&bitmap) {
            call(&bitmap, "close", &[])?;
        }
        set(&self.placeholder, "bitmap", &JsValue::UNDEFINED)?;
        let blueprint = create_canvas(&self.host.document()?, BLUEPRINT_WIDTH.into(), height)?;
        let Some(ctx) = context(&blueprint, Some(false))? else {
            return Ok(());
        };
        let data = create_image_data(&ctx, BLUEPRINT_WIDTH.into(), height, &JsValue::UNDEFINED)?;
        let buffer = get(&get(&data, "data")?, "buffer")?;
        let pixels = js_sys::Uint32Array::new(&buffer);
        // WASM and the source's RGBA8888 dependency both use little-endian bytes.
        pixels.fill(0xff00_0000, 0, pixels.length());
        for y in 0..pixels.length() / BLUEPRINT_WIDTH {
            for x in (0..BLUEPRINT_WIDTH).step_by(2) {
                pixels.set_index(y * BLUEPRINT_WIDTH + x + y % 2, 0xffff_ffff);
            }
        }
        call(&ctx, "putImageData", &[data, 0.into(), 0.into()])?;
        let aligned = int32(self.host.screen_width()? + f64::from(BLUEPRINT_WIDTH) - 1.0) & !31;
        let width = if aligned == 0 {
            PLACEHOLDER_LENGTH
        } else {
            aligned
        };
        let canvas = create_canvas(&self.host.document()?, width.into(), height)?;
        set(&self.placeholder, "canvas", &canvas)?;
        let Some(ctx) = context(&canvas, Some(false))? else {
            set(&self.placeholder, "canvas", &JsValue::UNDEFINED)?;
            return Ok(());
        };
        for x in (0..width).step_by(BLUEPRINT_WIDTH as usize) {
            call(&ctx, "drawImage", &[blueprint.clone(), x.into(), 0.into()])?;
        }
        assign_bitmap(&self.placeholder, &create_image_bitmap(&canvas)?)
    }
}
