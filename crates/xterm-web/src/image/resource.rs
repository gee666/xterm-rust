// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{
    js::{call, context, error, function, get, number},
    primitives::{create_canvas, create_image_data},
    BrowserImageRenderer,
};
use std::{any::Any, rc::Rc};
use wasm_bindgen::JsValue;
use xterm_addons::image::storage::{BitmapClose, ImageResource, ImageSurface, StorageError};

/// A real canvas or ImageBitmap. The creator specifies ownership: only a bitmap
/// has close semantics, and storage decides when its original resource closes.
pub struct BrowserImageSurface {
    value: JsValue,
    bitmap: bool,
}
impl BrowserImageSurface {
    pub fn canvas(value: JsValue) -> Self {
        Self {
            value,
            bitmap: false,
        }
    }
    pub fn bitmap(value: JsValue) -> Self {
        Self {
            value,
            bitmap: true,
        }
    }
    pub fn value(&self) -> JsValue {
        self.value.clone()
    }
    fn dimension(&self, key: &str) -> Result<u32, StorageError> {
        let value = number(&self.value, key).map_err(error)?;
        if !value.is_finite() || value < 0.0 || value > f64::from(u32::MAX) || value.fract() != 0.0
        {
            return Err(error(
                js_sys::TypeError::new(&format!("Invalid browser image {key}")).into(),
            ));
        }
        Ok(value as u32)
    }
}
impl ImageSurface for BrowserImageSurface {
    fn width(&self) -> Result<u32, StorageError> {
        self.dimension("width")
    }
    fn height(&self) -> Result<u32, StorageError> {
        self.dimension("height")
    }
    fn bitmap_close(&self) -> Result<Option<BitmapClose>, StorageError> {
        if !self.bitmap {
            return Ok(None);
        }
        let value = self.value.clone();
        let close = function(get(&value, "close").map_err(error)?, "close").map_err(error)?;
        Ok(Some(Rc::new(move || {
            close.call0(&value).map(|_| ()).map_err(error)
        })))
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl BrowserImageRenderer {
    pub(super) fn source(&self, resource: &ImageResource) -> Result<JsValue, JsValue> {
        if let Some(surface) = resource.surface::<BrowserImageSurface>() {
            return Ok(surface.value());
        }
        Err(js_sys::TypeError::new(
            "Upload decoded RGBA pixels before adding them to browser image storage",
        )
        .into())
    }

    /// Upload once at the protocol boundary. Storage then retains this canvas
    /// rather than allocating and copying the decoded image on every tile draw.
    pub fn upload_pixels(
        &self,
        pixels: &xterm_addons::image::types::RgbaImage,
    ) -> Result<ImageResource, JsValue> {
        let canvas = create_canvas(
            &self.host.document()?,
            pixels.width().into(),
            pixels.height().into(),
        )?;
        let ctx = context(&canvas, None)?
            .ok_or_else(|| js_sys::Error::new("Cannot create RGBA image canvas context"))?;
        let bytes = js_sys::Uint8Array::from(pixels.pixels());
        let data = create_image_data(
            &ctx,
            pixels.width().into(),
            pixels.height().into(),
            &bytes.buffer().into(),
        )?;
        call(&ctx, "putImageData", &[data, 0.into(), 0.into()])?;
        Ok(ImageResource::new(BrowserImageSurface::canvas(canvas)))
    }

    pub fn bitmap_to_canvas(&self, image: &ImageResource) -> Result<ImageResource, JsValue> {
        let source = self.source(image)?;
        let canvas = create_canvas(
            &self.host.document()?,
            number(&source, "width")?,
            number(&source, "height")?,
        )?;
        let ctx = context(&canvas, None)?
            .ok_or_else(|| js_sys::Error::new("Cannot create bitmap conversion canvas context"))?;
        call(&ctx, "drawImage", &[source, 0.into(), 0.into()])?;
        Ok(ImageResource::new(BrowserImageSurface::canvas(canvas)))
    }
}
