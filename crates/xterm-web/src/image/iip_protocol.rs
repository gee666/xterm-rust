// Copyright (c) 2023 The xterm.js authors. MIT license.
//! Streaming OSC1337 execution on the real browser terminal and shared store.
use super::{js, BrowserImageRenderer, IipDecodedImage, IipImageDecoder};
use crate::{
    js_error::{callback_error, to_js_error},
    terminal::{Shared, Terminal},
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::JsValue;
use xterm_addons::image::{
    iip::{
        geometry::{IipGeometry, RenderMetrics},
        handler::IipHandler,
        storage::IipImageStorage,
        types::{EndPreparation, GeometryPhase, IipEffect, IipOptions},
    },
    storage::{CellSize, ImageStorage},
    NameDecodePolicy,
};
use xterm_core::{
    lifecycle::DisposableHandle,
    parser::{
        handler::{DispatchError, HandlerResult},
        osc::{OscError, OscHandlerTrait},
    },
};

const IIP_OSC: f64 = 1337.0;
struct State {
    terminal: Weak<Shared>,
    renderer: Rc<BrowserImageRenderer>,
    storage: IipImageStorage,
    handler: RefCell<IipHandler>,
    decoder: IipImageDecoder,
}
fn dispatch(error: JsValue) -> DispatchError {
    DispatchError::Callback(callback_error(error))
}
fn warn(message: &str, cause: Option<JsValue>) -> Result<(), JsValue> {
    let console = js::get(&js_sys::global(), "console")?;
    let mut args = vec![message.into()];
    if let Some(cause) = cause {
        args.push(cause);
    }
    js::call(&console, "warn", &args)?;
    Ok(())
}
impl State {
    fn geometry(&self, phase: GeometryPhase) -> Result<IipGeometry, JsValue> {
        let shared = self
            .terminal
            .upgrade()
            .ok_or_else(|| js_sys::Error::new("IIP terminal was dropped"))?;
        let dimensions = self.renderer.host.dimensions()?;
        let services = shared.services.borrow().clone();
        let device_pixel_ratio = if phase == GeometryPhase::CellReport {
            services.map(|s| s.core_browser.dpr()).transpose()?
        } else {
            None
        };
        Ok(IipGeometry {
            columns: shared.native.cols().map_err(|e| to_js_error(&e))? as f64,
            rows: shared.native.rows().map_err(|e| to_js_error(&e))? as f64,
            render: dimensions.map(|d| RenderMetrics {
                cell: CellSize {
                    width: d.cell_width,
                    height: d.cell_height,
                },
                canvas: CellSize {
                    width: d.canvas_width,
                    height: d.canvas_height,
                },
            }),
            device_pixel_ratio,
        })
    }
    fn end(self: &Rc<Self>, success: bool) -> Result<HandlerResult, JsValue> {
        let preparation = self
            .handler
            .borrow_mut()
            .prepare_end(success)
            .map_err(|e| to_js_error(&e))?;
        let effects = match preparation {
            EndPreparation::Complete(effects) => effects,
            EndPreparation::Geometry(token) => {
                // Geometry can call browser code. End the controller borrow first,
                // then finish against live payload/header state after reentry.
                let geometry = self.geometry(token.phase())?;
                self.handler
                    .borrow_mut()
                    .finish_geometry(token, &geometry)
                    .map_err(|e| to_js_error(&e))?
            }
        };
        self.effects(effects.into_iter())
    }
    fn effects(
        self: &Rc<Self>,
        mut effects: std::vec::IntoIter<IipEffect>,
    ) -> Result<HandlerResult, JsValue> {
        while let Some(effect) = effects.next() {
            match effect {
                IipEffect::Report(text) => {
                    let shared = self
                        .terminal
                        .upgrade()
                        .ok_or_else(|| js_sys::Error::new("IIP terminal was dropped"))?;
                    shared
                        .native
                        .input(&text, false)
                        .map_err(|e| to_js_error(&e))?;
                }
                IipEffect::Warning(message) => {
                    IipHandler::report_warning(&self.handler, &message, |s| warn(s, None))?;
                }
                IipEffect::HostWarning { message, cause } => {
                    warn(&message, Some(to_js_error(cause.as_ref())))?;
                }
                IipEffect::Display(request) => {
                    // A pending source promise retains the terminal through its
                    // storage context, even after its parser token is disposed.
                    let terminal = self
                        .terminal
                        .upgrade()
                        .ok_or_else(|| js_sys::Error::new("IIP terminal was dropped"))?;
                    match self.decoder.prepare_transfer(&request, &self.handler)? {
                        IipDecodedImage::Immediate(image) => {
                            // QOI insertion failures are synchronous, outside the
                            // bitmap rejection catch in the source handler.
                            self.storage.add_image(image).map_err(|e| to_js_error(&e))?;
                        }
                        IipDecodedImage::Pending(pending) => {
                            let state = Rc::clone(self);
                            return Ok(HandlerResult::PendingFallible(Box::pin(async move {
                                let inserted = match pending.resolve().await {
                                    Ok(image) => {
                                        state.storage.add_image(image).map_err(|e| to_js_error(&e))
                                    }
                                    Err(error) => Err(error),
                                };
                                if let Err(error) = inserted {
                                    // Source catches both bitmap rejection and
                                    // storage exceptions from its then callback.
                                    let message = format!(
                                        "IIP: decoding error {} {}x{}",
                                        request.source.mime.mime(),
                                        request.source.width,
                                        request.source.height
                                    );
                                    warn(&message, Some(error)).map_err(dispatch)?;
                                }
                                let result =
                                    state.effects(effects).map_err(dispatch)?.resolve().await;
                                drop(terminal);
                                result
                            })));
                        }
                    }
                }
            }
        }
        Ok(HandlerResult::Ready(true))
    }
}

struct StreamingHandler(Rc<State>);
impl OscHandlerTrait for StreamingHandler {
    fn start(&mut self) -> Result<(), OscError> {
        self.0.handler.borrow_mut().start();
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), OscError> {
        self.0
            .handler
            .borrow_mut()
            .put(data, 0, data.len())
            .map_err(|e| dispatch(to_js_error(&e)))
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        self.0
            .end(success)
            .unwrap_or_else(|e| HandlerResult::Failed(dispatch(e)))
    }
}

/// IIP producer used by the image addon coordinator. This is not a partial public
/// ImageAddon: the coordinator still owns protocol selection and lifecycle hooks.
/// Registration disposal only unregisters. Reset does not cancel pending bitmap
/// promises, and those promises retain their original storage after disposal.
#[derive(Clone)]
pub struct BrowserIipProtocol(Rc<State>);
impl BrowserIipProtocol {
    pub fn new(
        terminal: &Terminal,
        renderer: Rc<BrowserImageRenderer>,
        storage: ImageStorage,
        options: IipOptions,
    ) -> Result<Self, JsValue> {
        Self::new_with_encoded_limit(
            terminal,
            renderer,
            storage,
            options,
            (options.size_limit * 4.0 / 3.0).ceil(),
        )
    }
    /// Use the host-evaluated cap without repeating observable JS conversion.
    pub fn new_with_encoded_limit(
        terminal: &Terminal,
        renderer: Rc<BrowserImageRenderer>,
        storage: ImageStorage,
        options: IipOptions,
        encoded_limit: f64,
    ) -> Result<Self, JsValue> {
        Ok(Self(Rc::new(State {
            terminal: Rc::downgrade(&terminal.shared),
            renderer,
            storage: IipImageStorage::new(storage),
            handler: RefCell::new(
                IipHandler::new_with_encoded_limit(
                    options,
                    NameDecodePolicy::BrowserAtob,
                    encoded_limit,
                )
                .map_err(|e| to_js_error(&e))?,
            ),
            decoder: IipImageDecoder::default(),
        })))
    }
    /// Register a streaming handler, avoiding the public whole-payload OSC limit.
    /// The caller owns the returned disposal token, as with the source parser.
    pub fn register(&self) -> Result<DisposableHandle, JsValue> {
        let terminal = self
            .0
            .terminal
            .upgrade()
            .ok_or_else(|| js_sys::Error::new("IIP terminal was dropped"))?;
        let (osc, _) = terminal
            .native
            .addon_registrars()
            .map_err(|e| to_js_error(&e))?;
        osc.register_handler(
            IIP_OSC,
            Rc::new(RefCell::new(StreamingHandler(Rc::clone(&self.0)))),
        )
        .ok_or_else(|| js_sys::Error::new("IIP parser was dropped").into())
    }
    pub fn reset(&self) {
        self.0.handler.borrow_mut().reset();
        self.0.decoder.reset();
    }
}
