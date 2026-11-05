// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Live addon ownership. Protocol registrations retain their original activation;
//! parent callbacks deliberately consult the current activation after reentry.
mod activation;
mod bindings;
mod controls;
mod handlers;
mod hooks;
mod inspection;
mod kitty_inspection;
mod option_inspection;
mod options;
mod protocols;
mod storage_limit;
use super::{
    BrowserIipProtocol, BrowserKittyProtocol, BrowserSixelProtocol, ImageRendererLifecycle,
};
use crate::terminal::Shared;
pub use bindings::ImageAddon;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use wasm_bindgen::JsValue;
use xterm_addons::image::{options::ImageOptions, storage::ImageStorage};
use xterm_core::{event::Emitter, lifecycle::Disposable};

type Cleanup = Rc<dyn Fn() -> Result<(), JsValue>>;
#[derive(Clone)]
enum Protocol {
    Sixel(BrowserSixelProtocol),
    Iip(BrowserIipProtocol),
    Kitty(BrowserKittyProtocol),
}
impl Protocol {
    fn reset(&self) -> Result<(), JsValue> {
        match self {
            Self::Sixel(protocol) => protocol.reset(),
            Self::Iip(protocol) => protocol.reset(),
            Self::Kitty(protocol) => return protocol.reset(),
        }
        Ok(())
    }
    fn key(&self) -> u8 {
        match self {
            Self::Sixel(_) => 0,
            Self::Iip(_) => 1,
            Self::Kitty(_) => 2,
        }
    }
}
struct State {
    options: Rc<Cell<ImageOptions>>,
    defaults: ImageOptions,
    storage_limit: Rc<RefCell<JsValue>>,
    placeholder: Rc<Cell<bool>>,
    // Raw public value is authoritative; the bool above is its storage policy.
    placeholder_value: Rc<RefCell<JsValue>>,
    terminal: RefCell<Weak<Shared>>,
    renderer: RefCell<Option<Rc<ImageRendererLifecycle>>>,
    storage: RefCell<Option<ImageStorage>>,
    inspection: RefCell<Option<JsValue>>,
    cleanups: RefCell<Vec<Cleanup>>,
    handlers: Rc<RefCell<handlers::Handlers>>,
    added: Emitter<()>,
}
impl State {
    fn storage(&self) -> Option<ImageStorage> {
        self.storage.borrow().clone()
    }
    fn renderer(&self) -> Option<Rc<ImageRendererLifecycle>> {
        self.renderer.borrow().clone()
    }
    fn reset_handlers(&self) -> Result<(), JsValue> {
        let mut index = 0;
        loop {
            let handler = self.handlers.borrow().at(index);
            let Some(handler) = handler else {
                return Ok(());
            };
            handler.reset()?;
            index += 1;
        }
    }
    fn dispose(&self) -> Result<(), JsValue> {
        let mut index = 0;
        loop {
            let cleanup = self.cleanups.borrow().get(index).cloned();
            let Some(cleanup) = cleanup else { break };
            cleanup()?;
            index += 1;
        }
        let cleanups = self.cleanups.take();
        let handlers = self.handlers.borrow_mut().clear();
        drop((cleanups, handlers));
        self.added.dispose();
        Ok(())
    }
}
