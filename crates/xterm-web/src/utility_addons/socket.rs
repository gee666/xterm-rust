// Copyright (c) 2014, 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Real browser WebSocket adapter for AttachAddon.
use crate::{
    utf16::{from_js_string, to_js_string},
    utility_addons::js_property,
};
use js_sys::{Array, JsString, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{Event, WebSocket};
use xterm_addons::attach::*;
pub struct BrowserSocket(pub WebSocket);

// Literal argument lists match source even if Array's iterator is replaced.
#[wasm_bindgen(
    inline_js = "export function attachAddListener(socket, type, handler) { socket.addEventListener(type, handler); } export function attachRemoveListener(socket, type, handler) { socket.removeEventListener(type, handler); }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = attachAddListener)]
    fn add_listener(socket: &JsValue, kind: &str, handler: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = attachRemoveListener)]
    fn remove_listener(socket: &JsValue, kind: &str, handler: &JsValue) -> Result<(), JsValue>;
}

fn error(value: JsValue) -> AttachError {
    AttachError::host(crate::js_error::JsCallbackError(value))
}
fn js_error(error: AttachError) -> JsValue {
    crate::js_error::to_js_error(&error)
}
impl AttachSocket for BrowserSocket {
    fn use_array_buffer(&self) -> Result<(), AttachError> {
        js_property::set(self.0.as_ref(), "binaryType", &"arraybuffer".into()).map_err(error)
    }
    fn state(&self) -> Result<SocketState, AttachError> {
        let value = js_property::get(self.0.as_ref(), "readyState")
            .map_err(error)?
            .as_f64();
        Ok(if value == Some(f64::from(WebSocket::OPEN)) {
            SocketState::Open
        } else if value == Some(f64::from(WebSocket::CONNECTING)) {
            SocketState::Connecting
        } else if value == Some(f64::from(WebSocket::CLOSING)) {
            SocketState::Closing
        } else if value == Some(f64::from(WebSocket::CLOSED)) {
            SocketState::Closed
        } else {
            SocketState::Unknown
        })
    }
    fn send(&self, data: SocketMessage) -> Result<(), AttachError> {
        let args = Array::new();
        match data {
            SocketMessage::Text(text) => {
                args.push(&to_js_string(&text));
            }
            // Own the browser array, since a user override of send may retain it.
            SocketMessage::Binary(bytes) => {
                args.push(&Uint8Array::from(bytes.as_slice()));
            }
            view @ SocketMessage::View(_) => {
                let source = view.into_parse_source()?;
                match source.read_all().map_err(AttachError::host)? {
                    xterm_core::input::parse_source::ParseChunk::Text(text) => {
                        args.push(&to_js_string(&text));
                    }
                    xterm_core::input::parse_source::ParseChunk::Bytes(bytes) => {
                        args.push(&Uint8Array::from(bytes.as_ref()));
                    }
                }
            }
        }
        js_property::call(self.0.as_ref(), "send", &args).map_err(error)?;
        Ok(())
    }
    fn listen(
        &self,
        event: SocketEvent,
        callback: SocketCallback,
    ) -> Result<AttachListener, AttachError> {
        let kind = match event {
            SocketEvent::Message => "message",
            SocketEvent::Close => "close",
            SocketEvent::Error => "error",
        };
        // Shared callbacks allow synchronous nested socket dispatch. FnMut's
        // exclusive WASM borrow would trap before the inner source callback.
        // Return Result so the JS exception is raised only after Rust cleanup.
        let listener = Closure::wrap(Box::new(move |value: Event| -> Result<(), JsValue> {
            let payload = if event == SocketEvent::Message {
                // Source uses property access, not same-realm instanceof.
                let data = js_property::get(value.as_ref(), "data")?;
                Some(if let Some(text) = data.dyn_ref::<JsString>() {
                    SocketMessage::Text(from_js_string(text))
                } else {
                    SocketMessage::View(crate::terminal::byte_view_payload(js_property::bytes(
                        &data,
                    )?))
                })
            } else {
                None
            };
            callback(payload).map_err(js_error)
        }) as Box<dyn Fn(Event) -> Result<(), JsValue>>);
        // JS owns the callback before registration: an override may retain it
        // and then throw, or removal may throw or deliberately do nothing.
        // into_js_value uses JS GC to keep the Rust closure alive for every
        // retained function reference, including an in-flight callback.
        let listener = listener.into_js_value();
        add_listener(self.0.as_ref(), kind, &listener).map_err(error)?;
        let socket = self.0.clone();
        Ok(AttachListener::new(move || {
            // Resolve the current method on each call with the source receiver
            // and exact two arguments. Never consume a handle on failure.
            remove_listener(socket.as_ref(), kind, &listener).map_err(error)
        }))
    }
    fn warn_closing(&self) -> Result<(), AttachError> {
        let console = js_property::get(&js_sys::global(), "console").map_err(error)?;
        let args = Array::new();
        args.push(&"Attach addon socket is closing".into());
        js_property::call(&console, "warn", &args).map_err(error)?;
        Ok(())
    }
}
