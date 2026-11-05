// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
// Streaming APC G execution uses the native protocol and actual browser codecs.
use super::{js, kitty_encoded::BlobFactory, kitty_placement, BrowserImageRenderer};
use crate::{
    js_error::to_js_error,
    terminal::{Shared, Terminal},
};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::JsValue;
use xterm_addons::image::{
    kitty::{
        handler::{
            DisplayFuture, KittyApcAdapter, KittyDisplayRequest, KittyGraphicsHandler,
            KittyGraphicsHost, KittyOptions,
        },
        storage::KittyImageStorage,
    },
    storage::{ImageStorage, StorageError},
};
use xterm_core::{
    lifecycle::DisposableHandle,
    parser::{
        apc::ApcHandlerTrait,
        escape::FunctionIdentifier,
        handler::{DispatchError, HandlerResult},
    },
};

struct RetainedAdapter {
    inner: KittyApcAdapter,
    terminal: Weak<Shared>,
}
impl ApcHandlerTrait for RetainedAdapter {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.inner.start()
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.inner.put(data)
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        let terminal = self.terminal.upgrade();
        let result = self.inner.end(success);
        match result {
            HandlerResult::Pending(_) | HandlerResult::PendingFallible(_) => {
                HandlerResult::PendingFallible(Box::pin(async move {
                    // Retain through the native controller's response callback,
                    // not merely through decode/placement. That callback runs
                    // after the host display future has already completed.
                    let result = result.resolve().await;
                    drop(terminal);
                    result
                }))
            }
            result => result,
        }
    }
}

struct Host {
    terminal: Weak<Shared>,
    renderer: Rc<BrowserImageRenderer>,
}
impl KittyGraphicsHost for Host {
    fn send_response(&self, data: &[u16]) -> Result<(), StorageError> {
        let terminal = self
            .terminal
            .upgrade()
            .ok_or_else(|| js::error(js_sys::Error::new("Kitty terminal was dropped").into()))?;
        terminal
            .native
            .input(data, false)
            .map_err(|e| js::error(to_js_error(&e)))
    }
    fn begin_display(&self, request: KittyDisplayRequest) -> DisplayFuture {
        let data = request.image.bytes.borrow().data.clone();
        let terminal = self.terminal.upgrade();
        let read = data.read();
        let renderer = Rc::clone(&self.renderer);
        Box::pin(async move {
            // A pending display retains its terminal/store after unregister,
            // reset or disposal, just like the source async instance method.
            let terminal = terminal.ok_or_else(|| {
                js::error(js_sys::Error::new("Kitty terminal was dropped").into())
            })?;
            kitty_placement::display(request, renderer, terminal, read)
                .await
                .map_err(js::error)
        })
    }
}

/// Producer for the full ImageAddon coordinator, not an incomplete public addon.
/// Registration disposal unregisters only. Explicit dispose follows source order:
/// Kitty storage first, handler second. Pending bitmap operations are not cancelled.
#[derive(Clone)]
pub struct BrowserKittyProtocol {
    terminal: Weak<Shared>,
    handler: Rc<KittyGraphicsHandler>,
    storage: Rc<KittyImageStorage>,
}
impl BrowserKittyProtocol {
    pub fn new(
        terminal: &Terminal,
        renderer: Rc<BrowserImageRenderer>,
        storage: ImageStorage,
        options: KittyOptions,
    ) -> Self {
        Self::new_with_encoded_limit(
            terminal,
            renderer,
            storage,
            options,
            (options.size_limit * 4.0 / 3.0).ceil(),
        )
    }
    /// Construct after the coordinator evaluates the raw JavaScript limit.
    pub(crate) fn new_with_encoded_limit(
        terminal: &Terminal,
        renderer: Rc<BrowserImageRenderer>,
        storage: ImageStorage,
        options: KittyOptions,
        max_encoded: f64,
    ) -> Self {
        let storage = Rc::new(KittyImageStorage::new(Rc::new(storage)));
        let host = Rc::new(Host {
            terminal: Rc::downgrade(&terminal.shared),
            renderer,
        });
        Self {
            terminal: Rc::downgrade(&terminal.shared),
            handler: Rc::new(KittyGraphicsHandler::new_with_encoded_limit(
                host,
                Rc::clone(&storage),
                Rc::new(BlobFactory),
                options,
                max_encoded,
            )),
            storage,
        }
    }
    pub fn register(&self) -> Result<DisposableHandle, JsValue> {
        let terminal = self
            .terminal
            .upgrade()
            .ok_or_else(|| js_sys::Error::new("Kitty terminal was dropped"))?;
        terminal
            .native
            .with_parser(|parser| {
                parser.register_streaming_apc_handler(
                    FunctionIdentifier::new("G"),
                    Rc::new(RefCell::new(RetainedAdapter {
                        inner: KittyApcAdapter::new(Rc::clone(&self.handler)),
                        terminal: self.terminal.clone(),
                    })),
                )
            })
            .map_err(|e| to_js_error(&e))?
            .map_err(|e| to_js_error(&e))
    }
    pub fn reset(&self) -> Result<(), JsValue> {
        self.handler.reset().map_err(|e| to_js_error(&e))
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.storage.dispose().map_err(|e| to_js_error(&e))?;
        self.handler.dispose().map_err(|e| to_js_error(&e))
    }
    pub fn handler(&self) -> Rc<KittyGraphicsHandler> {
        Rc::clone(&self.handler)
    }
}
