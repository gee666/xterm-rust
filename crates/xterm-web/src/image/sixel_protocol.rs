// Copyright (c) 2020, 2023 The xterm.js authors. MIT license.
//! Streaming SIXEL execution on the real browser terminal and shared store.
mod palette;
use super::{js, primitives::create_canvas, BrowserImageRenderer, BrowserImageSurface};
use crate::{
    js_error::{callback_error, to_js_error},
    terminal::{Shared, Terminal},
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use wasm_bindgen::{prelude::wasm_bindgen, JsValue};
use xterm_addons::image::{
    options::ImageOptions,
    sixel::{
        active_background, add_sixel_image, advance_sixel_cursor, ActiveColors, DecodeError,
        SixelDecoder, SixelEndPlan, SixelHandler, SixelPut, SixelWarning,
    },
    storage::{ImageResource, ImageStorage},
};
use xterm_core::{
    lifecycle::DisposableHandle,
    parser::{
        dcs::DcsHandlerTrait,
        escape::FunctionIdentifier,
        handler::{DispatchError, HandlerResult},
        params::Params,
    },
    terminal::line_feed_state,
};

#[wasm_bindgen(module = "/src/image/sixel_errors.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = sixelTrapText)]
    fn sixel_trap_text(kind: &str) -> Result<String, JsValue>;
}
#[wasm_bindgen(
    inline_js = "export function sixelSizeExceeded(size, limit) { return size > limit; }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = sixelSizeExceeded)]
    fn size_exceeded(size: f64, limit: &JsValue) -> Result<bool, JsValue>;
}
fn warning_text(warning: SixelWarning) -> Result<String, JsValue> {
    let kind = match warning {
        SixelWarning::Decoding(DecodeError::PaletteZero) => Some("remainder"),
        SixelWarning::Decoding(DecodeError::MemoryAccess) => Some("memory"),
        SixelWarning::Decoding(DecodeError::InvalidColor) => Some("signature"),
        _ => None,
    };
    match kind {
        Some(kind) => Ok(format!(
            "SIXEL: error while decoding image - {}",
            sixel_trap_text(kind)?
        )),
        None => Ok(warning.to_string()),
    }
}

struct State {
    terminal: Weak<Shared>,
    renderer: Rc<BrowserImageRenderer>,
    storage: ImageStorage,
    options: Rc<Cell<ImageOptions>>,
    // None keeps the native typed constructor's live option contract.
    size_limit: Option<JsValue>,
    palette: Option<RawPalette>,
    handler: RefCell<SixelHandler>,
}
struct RawPalette {
    current: Rc<dyn Fn() -> JsValue>,
    initial: JsValue,
}
fn dispatch(error: JsValue) -> DispatchError {
    DispatchError::Callback(callback_error(error))
}
impl State {
    fn terminal(&self) -> Result<Rc<Shared>, JsValue> {
        self.terminal
            .upgrade()
            .ok_or_else(|| js_sys::Error::new("SIXEL terminal was dropped").into())
    }
    fn background(&self) -> Result<u32, JsValue> {
        let terminal = self.terminal()?;
        let attributes = terminal
            .native
            .with_state(|s| s.attributes.clone())
            .map_err(|e| to_js_error(&e))?;
        let services = terminal.services.borrow().clone();
        let Some(services) = services else {
            return active_background(&attributes, None).map_err(|e| to_js_error(&e));
        };
        let colors = services.theme.colors();
        let colors = colors.borrow();
        let ansi: Vec<_> = colors.ansi.iter().map(|c| c.borrow().rgba).collect();
        let result = active_background(
            &attributes,
            Some(ActiveColors {
                foreground: colors.foreground.borrow().rgba,
                background: colors.background.borrow().rgba,
                ansi: &ansi,
            }),
        )
        .map_err(|e| to_js_error(&e));
        result
    }
    fn end(&self, success: bool) -> Result<(), JsValue> {
        let plan = self.handler.borrow().end_plan(success);
        match plan {
            SixelEndPlan::Handled => Ok(()),
            SixelEndPlan::AdvanceCursor { height } => advance_sixel_cursor(
                height as f64,
                self.options.get().sixel_scrolling,
                || self.renderer.cell_size(),
                || {
                    let terminal = self.terminal()?;
                    line_feed_state(&terminal.native.state_handle_for_addon())
                        .map_err(|e| to_js_error(&e))
                },
            ),
            SixelEndPlan::Image { width, height } => {
                // SIXEL creates in the global document, even for an adopted
                // terminal. It also inserts a blank canvas if getContext is null.
                let canvas = create_canvas(&JsValue::UNDEFINED, width as f64, height as f64)?;
                if let Some(context) = js::context(&canvas, None)? {
                    // JS reads the method and both constructors before data8.
                    // Getters may reset the live decoder or throw.
                    let put_image_data = js::get(&context, "putImageData")?;
                    let image_data = js::get(&js_sys::global(), "ImageData")?;
                    let clamped_array = js::get(&js_sys::global(), "Uint8ClampedArray")?;
                    let pixels = self.handler.borrow_mut().snapshot_image();
                    self.finish_pending_release()?;
                    let pixels = pixels
                        .map_err(|e| to_js_error(&e))?
                        .ok_or_else(|| js_sys::Error::new("SIXEL decoder was removed"))?;
                    let bytes = js_sys::Uint8Array::from(pixels.pixels());
                    let clamped = js_sys::Reflect::construct(
                        &js::function(clamped_array, "Uint8ClampedArray")?,
                        &js_sys::Array::of3(
                            &bytes.buffer(),
                            &0.into(),
                            &(pixels.pixels().len() as f64).into(),
                        ),
                    )?;
                    let image = js_sys::Reflect::construct(
                        &js::function(image_data, "ImageData")?,
                        &js_sys::Array::of3(
                            &clamped,
                            &(width as f64).into(),
                            &(height as f64).into(),
                        ),
                    )?;
                    js::function(put_image_data, "putImageData")?.call3(
                        &context,
                        &image,
                        &0.into(),
                        &0.into(),
                    )?;
                }
                // No decoder borrow crosses a canvas constructor, ImageData,
                // putImageData or storage callback. Reentry resets live state.
                let release = self
                    .handler
                    .borrow()
                    .needs_release_after_copy()
                    .map_err(|e| to_js_error(&e))?;
                if release {
                    self.release()?;
                }
                add_sixel_image(
                    &self.storage,
                    ImageResource::new(BrowserImageSurface::canvas(canvas)),
                    self.options.get().sixel_scrolling,
                )
                .map_err(|e| to_js_error(&e))
            }
        }
    }
}
struct StreamingHandler(Rc<State>);
impl DcsHandlerTrait for StreamingHandler {
    fn hook(&mut self, params: &Params) -> Result<(), DispatchError> {
        let ready = self.0.handler.borrow_mut().begin_hook();
        if ready {
            let fill = if params.stored_value(1) == Some(1) {
                0
            } else {
                self.0.background().map_err(dispatch)?
            };
            let limit = self.0.palette_limit().map_err(dispatch)?;
            self.0.handler.borrow_mut().finish_hook(fill, limit);
        }
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        let size = self.0.handler.borrow_mut().begin_put(data.len());
        let Some(size) = size else { return Ok(()) };
        // Source increments size before > coercion. Never hold the handler
        // borrow over user code, and do not repeat its guard after reentry.
        let exceeded = match &self.0.size_limit {
            Some(limit) => size_exceeded(size, limit).map_err(dispatch)?,
            None => size > self.0.options.get().sixel_size_limit,
        };
        let result = if exceeded {
            SixelPut::Warning(SixelWarning::TooMuchData)
        } else {
            self.0.handler.borrow_mut().decode_put(data)
        };
        if let SixelPut::Warning(warning) = result {
            let release_error = self.0.finish_pending_release().err();
            let console = js::get(&js_sys::global(), "console").map_err(dispatch)?;
            let warn = js::get(&console, "warn").map_err(dispatch)?;
            let text: JsValue = match release_error {
                Some(error) => palette::release_warning(&error).map_err(dispatch)?.into(),
                None => warning_text(warning).map_err(dispatch)?.into(),
            };
            js::function(warn, "console.warn")
                .map_err(dispatch)?
                .call1(&console, &text)
                .map_err(dispatch)?;
            self.0.handler.borrow_mut().mark_aborted();
            self.0.release().map_err(dispatch)?;
        }
        Ok(())
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        match self.0.end(success) {
            Ok(()) => HandlerResult::Ready(true),
            Err(error) => HandlerResult::Failed(dispatch(error)),
        }
    }
}

/// Live SIXEL producer, not a public ImageAddon constructor. The coordinator
/// owns the shared options, renderer, store, parser token and decoder readiness.
/// Start pending and deliver the decoder when asynchronous initialization
/// completes. Hook/put calls discarded while pending are never replayed.
#[derive(Clone)]
pub struct BrowserSixelProtocol(Rc<State>);
impl BrowserSixelProtocol {
    pub fn new(
        terminal: &Terminal,
        renderer: Rc<BrowserImageRenderer>,
        storage: ImageStorage,
        options: Rc<Cell<ImageOptions>>,
    ) -> Self {
        Self::create(terminal, renderer, storage, options, None, None)
    }
    /// Browser addon path. Preserve raw values until each ready packet's
    /// relational comparison, including BigInts and exact thrown values.
    pub fn new_with_size_limit(
        terminal: &Terminal,
        renderer: Rc<BrowserImageRenderer>,
        storage: ImageStorage,
        options: Rc<Cell<ImageOptions>>,
        size_limit: JsValue,
    ) -> Self {
        Self::create(terminal, renderer, storage, options, Some(size_limit), None)
    }
    pub(super) fn new_with_raw_options(
        terminal: &Terminal,
        renderer: Rc<BrowserImageRenderer>,
        storage: ImageStorage,
        options: Rc<Cell<ImageOptions>>,
        size_limit: JsValue,
        palette: Rc<dyn Fn() -> JsValue>,
    ) -> Self {
        let initial = palette();
        Self::create(
            terminal,
            renderer,
            storage,
            options,
            Some(size_limit),
            Some(RawPalette {
                current: palette,
                initial,
            }),
        )
    }
    fn create(
        terminal: &Terminal,
        renderer: Rc<BrowserImageRenderer>,
        storage: ImageStorage,
        options: Rc<Cell<ImageOptions>>,
        size_limit: Option<JsValue>,
        palette: Option<RawPalette>,
    ) -> Self {
        Self(Rc::new(State {
            terminal: Rc::downgrade(&terminal.shared),
            renderer,
            storage,
            options,
            size_limit,
            palette,
            handler: RefCell::new(SixelHandler::pending()),
        }))
    }
    pub fn complete_loading(&self, mut decoder: SixelDecoder) {
        if self.0.palette.is_some() {
            decoder.defer_memory_release();
        }
        self.0.handler.borrow_mut().complete_loading(decoder);
    }
    pub fn register(&self) -> Result<DisposableHandle, JsValue> {
        self.0
            .terminal()?
            .native
            .with_parser(|parser| {
                parser.register_streaming_dcs_handler(
                    FunctionIdentifier {
                        prefix: None,
                        intermediates: None,
                        final_byte: "q",
                    },
                    Rc::new(RefCell::new(StreamingHandler(Rc::clone(&self.0)))),
                )
            })
            .map_err(|e| to_js_error(&e))?
            .map_err(|e| to_js_error(&e))
    }
    /// Reset the typed producer using its live native palette option.
    pub fn reset(&self) {
        self.0
            .handler
            .borrow_mut()
            .reset(self.0.options.get().sixel_palette_limit);
    }
    pub(super) fn reset_raw(&self) -> Result<(), JsValue> {
        self.0.reset_palette()
    }
}
