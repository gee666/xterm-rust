// Copyright (c) 2023 The xterm.js authors. MIT license.
//! IIP image decoding at the browser boundary. Protocol decisions stay in Rust.
use super::{
    iip_memory::QoiMemory,
    js::{call, construct, context, function, get, set},
    primitives::create_canvas,
    BrowserImageSurface,
};
use js_sys::{Array, Promise, Uint8Array};
use std::cell::RefCell;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use xterm_addons::image::{
    iip::{
        handler::IipHandler,
        types::{DisplayRequest, DECODER_KEEP_BYTES},
    },
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
    memory: QoiMemory,
}
struct QoiSnapshot {
    range: Result<std::ops::Range<usize>, QoiError>,
    memory: Vec<u8>,
    width: i32,
    height: i32,
}
impl Default for IipImageDecoder {
    fn default() -> Self {
        Self {
            qoi: RefCell::new(QoiDecoder::new(DECODER_KEEP_BYTES as f64)),
            memory: QoiMemory::default(),
        }
    }
}
impl IipImageDecoder {
    pub fn reset(&self) {
        self.release();
    }
    fn release(&self) -> (i32, i32) {
        let (initialized, width, height) = {
            let mut qoi = self.qoi.borrow_mut();
            qoi.release();
            (qoi.is_initialized(), qoi.width, qoi.height)
        };
        if !initialized {
            self.memory.clear();
        }
        (width, height)
    }
    fn decode_qoi(&self, input: &[u8], previous: Option<Vec<u8>>) -> Result<QoiSnapshot, QoiError> {
        let mut qoi = self.qoi.borrow_mut();
        if let Some(previous) = previous {
            if previous.len() != qoi.memory_usage() {
                return Err(QoiError::InputOffset);
            }
            qoi.memory_mut().copy_from_slice(&previous);
        }
        let range = qoi.decode_in_place(input);
        // Snapshot before browser calls, without unsafe WASM memory views or
        // retained RefCell borrows. The browser mirror itself remains shared.
        let mut memory = Vec::new();
        memory.try_reserve_exact(qoi.memory_usage())?;
        memory.extend_from_slice(qoi.memory());
        Ok(QoiSnapshot {
            range,
            memory,
            width: qoi.width,
            height: qoi.height,
        })
    }

    /// Prepare synchronously so QOI/ImageData/Blob and createImageBitmap call
    /// failures are not mistaken for rejected bitmap promises. The protocol
    /// handler already validates metrics and target dimensions before this call.
    /// This standalone operation owns no protocol state. A live IIP coordinator
    /// must use `prepare_transfer` to preserve BASE64 release and failure timing.
    pub fn prepare(&self, request: &DisplayRequest) -> Result<IipDecodedImage, JsValue> {
        self.prepare_image(request, None)
    }

    /// Prepare a real protocol transfer. No handler borrow crosses a browser
    /// call. Source releases the live BASE64 decoder only after ImageData/QOI
    /// preparation or Blob construction succeeds, including on nested transfer.
    pub fn prepare_transfer(
        &self,
        request: &DisplayRequest,
        handler: &RefCell<IipHandler>,
    ) -> Result<IipDecodedImage, JsValue> {
        self.prepare_image(request, Some(handler))
    }

    fn prepare_image(
        &self,
        request: &DisplayRequest,
        handler: Option<&RefCell<IipHandler>>,
    ) -> Result<IipDecodedImage, JsValue> {
        let image = if request.source.mime == ImageType::Qoi {
            let previous = self.memory.snapshot()?;
            let snapshot = self
                .decode_qoi(&request.bytes, previous)
                .map_err(qoi_error)?;
            let buffer = self.memory.update(&snapshot.memory)?;
            let range = snapshot.range.map_err(qoi_error)?;
            let clamped = construct(
                "Uint8ClampedArray",
                &[buffer, range.start.into(), range.len().into()],
            )?;
            let image = construct(
                "ImageData",
                &[clamped, snapshot.width.into(), snapshot.height.into()],
            )?;
            // ImageData can synchronously decode again into the same memory.
            // Preserve that aliasing, then read live dimensions after release.
            let (width, height) = self.release();
            if request.target.width == f64::from(width)
                && request.target.height == f64::from(height)
            {
                release_payload(handler);
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
        release_payload(handler);
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

fn release_payload(handler: Option<&RefCell<IipHandler>>) {
    if let Some(handler) = handler {
        handler.borrow_mut().release_payload();
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
