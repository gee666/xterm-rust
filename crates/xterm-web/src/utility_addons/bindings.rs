// Copyright (c) 2014, 2019, 2023 The xterm.js authors. MIT license.
//! Callable attach and base64 exports over the Rust-backed browser Terminal.
pub use super::clipboard_bindings::ClipboardAddon;
use super::{attach_terminal::BrowserAttachTerminal, socket::BrowserSocket};
use crate::terminal::Terminal;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_addons::{attach, native_utility::NativeUtilityTerminal};

#[wasm_bindgen(js_name=AttachAddon)]
pub struct AttachAddon {
    addon: attach::AttachAddon,
    terminals: RefCell<Vec<Rc<BrowserAttachTerminal>>>,
}
#[wasm_bindgen]
impl AttachAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(socket: web_sys::WebSocket, options: JsValue) -> Result<AttachAddon, JsValue> {
        let addon = attach::AttachAddon::with_options(Rc::new(BrowserSocket(socket)), move || {
            if !options.is_truthy() {
                return Ok(None);
            }
            let value = super::js_property::get(&options, "bidirectional").map_err(|error| {
                attach::AttachError::host(crate::js_error::JsCallbackError(error))
            })?;
            Ok(Some(value.as_bool() != Some(false)))
        })
        .map_err(|error| crate::js_error::to_js_error(&error))?;
        Ok(Self {
            addon,
            terminals: RefCell::default(),
        })
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let native = NativeUtilityTerminal::from_headless(terminal.shared_native())
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        let host = Rc::new(BrowserAttachTerminal {
            native: Rc::new(native),
            writer: terminal.queued_write_handle(),
        });
        self.terminals.borrow_mut().push(host.clone());
        self.addon
            .activate(host)
            .map_err(|error| crate::js_error::to_js_error(&error))
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.addon
            .try_dispose()
            .map_err(|error| crate::js_error::to_js_error(&error))
    }
}
#[wasm_bindgen(js_name=Base64)]
pub struct Base64;
#[wasm_bindgen]
impl Base64 {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Base64 {
        Self
    }
    #[wasm_bindgen(js_name=encodeText)]
    pub fn encode_text(&self, value: JsValue) -> Result<JsValue, JsValue> {
        super::clipboard_bindings::encode_default(&value)
    }
    #[wasm_bindgen(js_name=decodeText)]
    pub fn decode_text(&self, value: JsValue) -> Result<JsValue, JsValue> {
        super::clipboard_bindings::decode_default(&value)
    }
}
impl Default for Base64 {
    fn default() -> Self {
        Self::new()
    }
}
