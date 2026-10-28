// Copyright (c) 2014, 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Real browser WebSocket adapter for AttachAddon.
use crate::utf16::{from_js_string, to_js_string};
use js_sys::{ArrayBuffer, Function, JsString, Reflect, Uint8Array};
use std::cell::RefCell;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{BinaryType, Event, MessageEvent, WebSocket};
use xterm_addons::attach::*;
use xterm_core::lifecycle::{to_disposable, DisposableHandle};

pub struct BrowserSocket(pub WebSocket);
fn error(value: JsValue) -> AttachError {
    AttachError::host(crate::js_error::JsCallbackError(value))
}
fn js_error(error: AttachError) -> JsValue {
    crate::js_error::to_js_error(&error)
}
impl AttachSocket for BrowserSocket {
    fn use_array_buffer(&self) -> Result<(), AttachError> {
        self.0.set_binary_type(BinaryType::Arraybuffer);
        Ok(())
    }
    fn state(&self) -> SocketState {
        match self.0.ready_state() {
            WebSocket::CONNECTING => SocketState::Connecting,
            WebSocket::OPEN => SocketState::Open,
            WebSocket::CLOSING => SocketState::Closing,
            WebSocket::CLOSED => SocketState::Closed,
            _ => SocketState::Unknown,
        }
    }
    fn send(&self, data: SocketMessage) -> Result<(), AttachError> {
        match data {
            SocketMessage::Text(text) => {
                // send_with_str cannot preserve unpaired UTF-16 surrogates.
                let method = Reflect::get(self.0.as_ref(), &"send".into())
                    .map_err(error)?
                    .dyn_into::<Function>()
                    .map_err(error)?;
                method
                    .call1(self.0.as_ref(), &to_js_string(&text))
                    .map_err(error)?;
                Ok(())
            }
            SocketMessage::Binary(bytes) => self.0.send_with_u8_array(&bytes).map_err(error),
        }
    }
    fn listen(
        &self,
        event: SocketEvent,
        callback: SocketCallback,
    ) -> Result<DisposableHandle, AttachError> {
        let kind = match event {
            SocketEvent::Message => "message",
            SocketEvent::Close => "close",
            SocketEvent::Error => "error",
        };
        // Return Result from the closure so wasm-bindgen throws only after Rust
        // locals are dropped. throw_val inside the closure would skip cleanup.
        let listener = Closure::wrap(Box::new(move |event_value: Event| -> Result<(), JsValue> {
            let payload = if event == SocketEvent::Message {
                let event = event_value.dyn_into::<MessageEvent>()?;
                let data = event.data();
                Some(if let Some(text) = data.dyn_ref::<JsString>() {
                    SocketMessage::Text(from_js_string(text))
                } else if data.is_instance_of::<ArrayBuffer>() {
                    SocketMessage::Binary(Uint8Array::new(&data).to_vec())
                } else {
                    return Err(js_error(AttachError::Adapter(
                        "WebSocket delivered neither string nor ArrayBuffer".into(),
                    )));
                })
            } else {
                None
            };
            callback(payload).map_err(js_error)
        }) as Box<dyn FnMut(Event) -> Result<(), JsValue>>);
        self.0
            .add_event_listener_with_callback(kind, listener.as_ref().unchecked_ref())
            .map_err(error)?;
        let listener = RefCell::new(Some(listener));
        let socket = self.0.clone();
        Ok(to_disposable(move || {
            if let Some(callback) = listener.borrow_mut().take() {
                // The target/type/callback tuple is unchanged; built-in removal
                // is non-throwing for this validated registration.
                socket
                    .remove_event_listener_with_callback(kind, callback.as_ref().unchecked_ref())
                    .expect("valid built-in WebSocket listener removal");
            }
        }))
    }
    fn warn_closing(&self) {
        web_sys::console::warn_1(&"Attach addon socket is closing".into());
    }
}
