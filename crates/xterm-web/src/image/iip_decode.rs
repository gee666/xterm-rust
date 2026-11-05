// Copyright (c) 2023 The xterm.js authors. MIT license.
//! IIP image decoding at the browser boundary. Protocol decisions stay in Rust.
use super::{
    js::{call, construct, context, function, get, set},
    primitives::create_canvas,
    BrowserImageSurface,
};
use js_sys::{Array, Promise, Uint8Array};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use xterm_addons::image::{
    iip::types::{DisplayRequest, DECODER_KEEP_BYTES},
    metrics::ImageType,
    qoi::{QoiDecoder, QoiError},
    storage::ImageResource,
};

/// The unscaled QOI path inserts its canvas synchronously upstream. Matching it
/// requires the caller to handle `Immediate` without scheduling a future.
pub enum IipDecodedImage {
    Immediate(ImageResource),
    Pending(PendingImageBitmap),
}

/// Owns the browser promise, not the protocol decoder or a terminal borrow.
/// Reset/disposal must not cancel this promise: source insertions can finish late.
pub struct PendingImageBitmap(Promise);
impl PendingImageBitmap {
    /// Rejections remain errors. The IIP coordinator decides when to warn, and
    /// must include failures from storage insertion in the same rejection path.
    pub async fn resolve(self) -> Result<ImageResource, JsValue> {
        let value = JsFuture::from(self.0).await?;
        Ok(ImageResource::new(BrowserImageSurface::bitmap(value)))
    }
}

/// Retains QOI memory across transfers, following the source handler's lifetime.
/// This consumes actual IIP display requests, never a JavaScript protocol engine.
pub struct IipImageDecoder {
    qoi: RefCell<QoiDecoder>,
}
impl Default for IipImageDecoder {
    fn default() -> Self {
        Self {
            qoi: RefCell::new(QoiDecoder::new(DECODER_KEEP_BYTES as f64)),
        }
    }
}
impl IipImageDecoder {
    pub fn reset(&self) {
        self.qoi.borrow_mut().release();
    }

    /// Prepare synchronously so QOI/ImageData/Blob and createImageBitmap call
    /// failures are not mistaken for rejected bitmap promises. The protocol
    /// handler already validates metrics and target dimensions before this call.
    pub fn prepare(&self, request: &DisplayRequest) -> Result<IipDecodedImage, JsValue> {
        let image = if request.source.mime == ImageType::Qoi {
            // Native decode has no foreign callbacks. End its borrow before even
            // constructing a JS error, so reset/transfer reentry stays possible.
            let result = self.qoi.borrow_mut().decode(&request.bytes);
            let decoded = result.map_err(qoi_error)?;
            let bytes = Uint8Array::from(decoded.pixels.as_slice());
            let clamped = construct(
                "Uint8ClampedArray",
                &[bytes.buffer().into(), 0.into(), bytes.length().into()],
            )?;
            let image = construct(
                "ImageData",
                &[clamped, decoded.width.into(), decoded.height.into()],
            )?;
            // ImageData must succeed before release, including for malformed QOI.
            let (width, height) = {
                let mut qoi = self.qoi.borrow_mut();
                qoi.release();
                // Reentrant ImageData construction may have changed the decoder.
                // Source checks its live dimensions after that callback returns.
                (qoi.width, qoi.height)
            };
            if request.target.width == f64::from(width)
                && request.target.height == f64::from(height)
            {
                let canvas = create_canvas(&JsValue::UNDEFINED, width.into(), height.into())?;
                if let Some(context) = context(&canvas, None)? {
                    call(&context, "putImageData", &[image, 0.into(), 0.into()])?;
                }
                return Ok(IipDecodedImage::Immediate(ImageResource::new(
                    BrowserImageSurface::canvas(canvas),
                )));
            }
            image
        } else {
            let parts = Array::new();
            parts.push(&Uint8Array::from(request.bytes.as_slice()));
            let options = js_sys::Object::new();
            set(&options, "type", &request.source.mime.mime().into())?;
            construct("Blob", &[parts.into(), options.into()])?
        };
        let options = js_sys::Object::new();
        set(&options, "resizeWidth", &request.target.width.into())?;
        set(&options, "resizeHeight", &request.target.height.into())?;
        let promise = function(
            get(&js_sys::global(), "createImageBitmap")?,
            "createImageBitmap",
        )?
        .call2(&JsValue::UNDEFINED, &image, &options)?
        .dyn_into::<Promise>()
        .map_err(|_| js_sys::TypeError::new("createImageBitmap did not return a Promise"))?;
        Ok(IipDecodedImage::Pending(PendingImageBitmap(promise)))
    }
}

fn qoi_error(error: QoiError) -> JsValue {
    let message = error.to_string();
    match error.js_name() {
        "RuntimeError" => js_sys::WebAssembly::RuntimeError::new(&message).into(),
        "LinkError" => js_sys::WebAssembly::LinkError::new(&message).into(),
        "TypeError" => js_sys::TypeError::new(&message).into(),
        _ => js_sys::RangeError::new(&message).into(),
    }
}
