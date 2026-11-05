// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::{
    js,
    kitty_encoded::{await_value, bitmap_call, byte_view},
};
use std::rc::Rc;
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use xterm_addons::image::{
    encoded::ByteRead,
    kitty::{
        pixels::{raw_pixels, RawFormat},
        storage::KittyImageData,
    },
};

#[wasm_bindgen(module = "/src/image/kitty_host.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = pngBitmapFallback)]
    fn png_fallback(blob: &JsValue) -> Result<JsValue, JsValue>;
}

pub(super) async fn decode(image: Rc<KittyImageData>, read: ByteRead) -> Result<JsValue, JsValue> {
    let mut bytes = read.await.map_err(|e| crate::js_error::to_js_error(&e))?;
    let compressed = image.bytes.borrow().compression == [u16::from(b'z')];
    if compressed {
        bytes = match decompress(&bytes, "deflate").await {
            Ok(bytes) => bytes,
            // The source specifically retries raw DEFLATE after zlib failure.
            Err(_) => decompress(&bytes, "deflate-raw").await?,
        };
    }
    let metadata = image.bytes.borrow().clone();
    let input = if metadata.format == 100.0 {
        let parts = js_sys::Array::new();
        parts.push(&js_sys::Uint8Array::from(bytes.as_slice()));
        let options = js_sys::Object::new();
        js::set(&options, "type", &"image/png".into())?;
        let blob = js::construct("Blob", &[parts.into(), options.into()])?;
        let window = js::get(&js_sys::global(), "window")?;
        if !js::get(&window, "createImageBitmap")?.is_truthy() {
            return await_value(png_fallback(&blob)?).await;
        }
        blob
    } else {
        let format = if metadata.format == 32.0 {
            RawFormat::Rgba
        } else {
            RawFormat::Rgb
        };
        let pixels = raw_pixels(&bytes, metadata.width, metadata.height, format)
            .map_err(|e| js_sys::Error::new(&e.to_string()))?;
        let clamped = js::construct(
            "Uint8ClampedArray",
            &[js_sys::Uint8Array::from(pixels.pixels()).buffer().into()],
        )?;
        js::construct(
            "ImageData",
            &[clamped, metadata.width.into(), metadata.height.into()],
        )?
    };
    await_value(bitmap_call(&[input])?).await
}

async fn decompress(bytes: &[u8], format: &str) -> Result<Vec<u8>, JsValue> {
    let stream = js::construct("DecompressionStream", &[format.into()])?;
    let writer = js::call(&js::get(&stream, "writable")?, "getWriter", &[])?;
    // Source queues both operations before reading, without awaiting writes.
    js::call(&writer, "write", &[js_sys::Uint8Array::from(bytes).into()])?;
    js::call(&writer, "close", &[])?;
    let reader = js::call(&js::get(&stream, "readable")?, "getReader", &[])?;
    let mut output = Vec::new();
    loop {
        let result = await_value(js::call(&reader, "read", &[])?).await?;
        if js::get(&result, "done")?.is_truthy() {
            break;
        }
        let bytes = byte_view(&js::get(&result, "value")?)?;
        output
            .try_reserve(bytes.len())
            .map_err(|e| js_sys::RangeError::new(&e.to_string()))?;
        output.extend_from_slice(&bytes);
    }
    Ok(output)
}
