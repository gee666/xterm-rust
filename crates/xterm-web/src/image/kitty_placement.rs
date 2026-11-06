// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::{
    js, kitty_decode,
    kitty_encoded::{await_value, bitmap_call},
    pixel_limit::PixelLimit,
    primitives::create_canvas,
    BrowserImageRenderer, BrowserImageSurface,
};
use crate::{js_error::to_js_error, terminal::Shared};
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_addons::image::{
    encoded::ByteRead,
    kitty::{
        command::KittyCommand,
        geometry::{self, BitmapSize},
        handler::KittyDisplayRequest,
    },
    storage::{CellSize, ImageResource},
};

fn command(request: &KittyDisplayRequest) -> KittyCommand {
    request.command.borrow().clone()
}
fn size(bitmap: &JsValue) -> Result<BitmapSize, JsValue> {
    fn dimension(bitmap: &JsValue, key: &str) -> Result<u32, JsValue> {
        let value = js::number(bitmap, key)?;
        if !value.is_finite() || value < 0.0 || value > f64::from(u32::MAX) || value.fract() != 0.0
        {
            return Err(js_sys::TypeError::new("Invalid bitmap dimensions").into());
        }
        Ok(value as u32)
    }
    Ok(BitmapSize {
        width: dimension(bitmap, "width")?,
        height: dimension(bitmap, "height")?,
    })
}
fn cursor_index(value: f64) -> Result<usize, JsValue> {
    if !value.is_finite() || value < 0.0 || value > usize::MAX as f64 || value.fract() != 0.0 {
        return Err(js_sys::RangeError::new("Invalid Kitty cursor position").into());
    }
    Ok(value as usize)
}
fn protocol_error(error: impl std::fmt::Display) -> JsValue {
    js_sys::Error::new(&error.to_string()).into()
}

/// The decoded bitmap is owned until storage accepts it. Browser failures close
/// exactly the current resource, including a close failure overriding the cause.
pub(super) async fn display(
    request: KittyDisplayRequest,
    renderer: Rc<BrowserImageRenderer>,
    terminal: Rc<Shared>,
    read: ByteRead,
    pixel_limit: PixelLimit,
) -> Result<(), JsValue> {
    let decoded = kitty_decode::decode(Rc::clone(&request.image), read).await?;
    let mut bitmap = Some(decoded);
    let result = place(&request, &renderer, &terminal, &mut bitmap, &pixel_limit).await;
    if result.is_err() {
        if let Some(bitmap) = bitmap.filter(|value| !js::nullish(value)) {
            js::call(&bitmap, "close", &[])?;
        }
    }
    result
}
async fn replace(bitmap: &mut Option<JsValue>, args: &[JsValue]) -> Result<(), JsValue> {
    let replacement = await_value(bitmap_call(args)?).await?;
    if let Some(old) = bitmap.as_ref() {
        js::call(old, "close", &[])?;
    }
    *bitmap = Some(replacement);
    Ok(())
}
async fn place(
    request: &KittyDisplayRequest,
    renderer: &BrowserImageRenderer,
    terminal: &Shared,
    bitmap: &mut Option<JsValue>,
    pixel_limit: &PixelLimit,
) -> Result<(), JsValue> {
    // Every call here has a live bitmap until ownership transfer below.
    let current = bitmap
        .as_ref()
        .ok_or_else(|| js_sys::Error::new("Missing Kitty bitmap"))?;
    let crop = geometry::crop(size(current)?, &command(request)).map_err(protocol_error)?;
    if crop.required {
        replace(
            bitmap,
            &[
                current.clone(),
                crop.x.into(),
                crop.y.into(),
                crop.width.into(),
                crop.height.into(),
            ],
        )
        .await?;
    }
    let measured = renderer
        .host
        .dimensions()?
        .map(|d| CellSize {
            width: d.cell_width,
            height: d.cell_height,
        })
        .unwrap_or(CellSize {
            width: 0.0,
            height: 0.0,
        });
    let current = bitmap
        .as_ref()
        .ok_or_else(|| js_sys::Error::new("Missing Kitty bitmap"))?;
    let actual = size(current)?;
    let mut layout = geometry::prepare_layout(actual, measured, &command(request));
    pixel_limit.check(layout.width, layout.height)?;
    // Source captures the actual buffer object before any scaling await.
    let buffer = terminal
        .native
        .with_state(|state| state.active_buffer())
        .map_err(|e| to_js_error(&e))?;
    let (saved_x, saved_y, saved_base) = {
        let b = buffer.borrow();
        (b.x as f64, b.y as f64, b.ybase as f64)
    };
    if layout.width != f64::from(actual.width) || layout.height != f64::from(actual.height) {
        let options = js_sys::Object::new();
        js::set(&options, "resizeWidth", &layout.width.into())?;
        js::set(&options, "resizeHeight", &layout.height.into())?;
        replace(bitmap, &[current.clone(), options.into()]).await?;
    }
    let current = bitmap
        .as_ref()
        .ok_or_else(|| js_sys::Error::new("Missing Kitty bitmap"))?;
    if let Some(offset) = geometry::offset(size(current)?, layout, &command(request)) {
        let window = js::get(&js_sys::global(), "window")?;
        let canvas = create_canvas(
            &js::get(&window, "document")?,
            offset.canvas_width,
            offset.canvas_height,
        )?;
        let context = js::context(&canvas, None)?
            .ok_or_else(|| js_sys::Error::new("Failed to create offset canvas context"))?;
        js::call(
            &context,
            "drawImage",
            &[current.clone(), offset.x.into(), offset.y.into()],
        )?;
        let replacement = await_value(bitmap_call(std::slice::from_ref(&canvas))?).await?;
        // Chained source assignment sets height before width.
        js::set(&canvas, "height", &0.into())?;
        js::set(&canvas, "width", &0.into())?;
        js::call(current, "close", &[])?;
        *bitmap = Some(replacement);
        let current = bitmap
            .as_ref()
            .ok_or_else(|| js_sys::Error::new("Missing Kitty bitmap"))?;
        let actual = size(current)?;
        pixel_limit.check(f64::from(actual.width), f64::from(actual.height))?;
        layout = geometry::finish_offset(actual, layout, &command(request));
    }
    let current = bitmap
        .as_ref()
        .ok_or_else(|| js_sys::Error::new("Missing Kitty bitmap"))?;
    let options = geometry::placement_options(layout, &command(request));
    request
        .storage
        .add_image(
            request.image.id.get(),
            ImageResource::new(BrowserImageSurface::bitmap(current.clone())),
            options.scrolling,
            options.layer,
            options.z_index,
        )
        .map_err(|e| to_js_error(&e))?;
    *bitmap = None;
    let current_base = buffer.borrow().ybase as f64;
    let columns = terminal.native.cols().map_err(|e| to_js_error(&e))? as f64;
    let cursor = geometry::cursor(
        &command(request),
        saved_x,
        saved_y,
        saved_base,
        current_base,
        columns,
        layout.columns,
    );
    let x = cursor_index(cursor.x)?;
    let y = cursor.y.map(cursor_index).transpose()?;
    let mut buffer = buffer.borrow_mut();
    buffer.x = x;
    if let Some(y) = y {
        buffer.y = y;
    }
    Ok(())
}
