//! Built-in mouse modes expose the same service used by parsing and DOM input.
//! Raw built-in fields use the shared Rust policies; custom registries remain
//! a separate, unfinished adapter.
use super::events;
use crate::js_error::to_js_error;
use js_sys::Function;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use xterm_core::{
    lifecycle::Disposable,
    mouse_fields::{Encoding, Protocol},
    mouse_state::MouseStateService,
};
use xterm_wasm_support::{mouse_fields::RawMouseEvent, utf16::to_js_string};

#[wasm_bindgen]
pub struct BrowserMouseState {
    service: Rc<MouseStateService>,
    on_protocol_change: Function,
}

impl BrowserMouseState {
    pub(super) fn new(service: Rc<MouseStateService>) -> Self {
        let on_protocol_change = events::callable(service.on_protocol_change(), |events| {
            Ok(JsValue::from(events.0))
        });
        Self {
            service,
            on_protocol_change,
        }
    }
}

#[wasm_bindgen]
impl BrowserMouseState {
    #[wasm_bindgen(getter, js_name = activeProtocol)]
    pub fn active_protocol(&self) -> String {
        self.service.active_protocol()
    }

    /// Built-in string keys share native validation and protocol-change events.
    #[wasm_bindgen(setter, js_name = activeProtocol)]
    pub fn set_active_protocol(&self, name: &str) -> Result<(), JsValue> {
        self.service
            .set_active_protocol(name)
            .map_err(|error| to_js_error(&error))
    }

    #[wasm_bindgen(getter, js_name = activeEncoding)]
    pub fn active_encoding(&self) -> String {
        self.service.active_encoding()
    }

    #[wasm_bindgen(setter, js_name = activeEncoding)]
    pub fn set_active_encoding(&self, name: &str) -> Result<(), JsValue> {
        self.service
            .set_active_encoding(name)
            .map_err(|error| to_js_error(&error))
    }

    #[wasm_bindgen(getter, js_name = areMouseEventsActive)]
    pub fn are_mouse_events_active(&self) -> bool {
        self.service.are_mouse_events_active()
    }

    #[wasm_bindgen(getter, js_name = isDefaultEncoding)]
    pub fn is_default_encoding(&self) -> bool {
        self.service.is_default_encoding()
    }

    #[wasm_bindgen(getter, js_name = isPixelEncoding)]
    pub fn is_pixel_encoding(&self) -> bool {
        self.service.is_pixel_encoding()
    }

    #[wasm_bindgen(getter, js_name = onProtocolChange)]
    pub fn on_protocol_change(&self) -> Function {
        self.on_protocol_change.clone()
    }

    /// Read the native active mode before touching the event. A field accessor
    /// may switch modes recursively, but the current call keeps its policy.
    #[wasm_bindgen(js_name = restrictMouseEvent)]
    pub fn restrict_mouse_event(&self, event: JsValue) -> Result<bool, JsValue> {
        let policy = match self.service.active_protocol().as_str() {
            "NONE" => Protocol::None,
            "X10" => Protocol::X10,
            "VT200" => Protocol::Vt200,
            "DRAG" => Protocol::Drag,
            "ANY" => Protocol::Any,
            name => {
                return Err(js_sys::Error::new(&format!(
                    "unsupported raw mouse protocol \"{name}\""
                ))
                .into())
            }
        };
        policy.restrict(&mut RawMouseEvent(event))
    }

    #[wasm_bindgen(js_name = encodeMouseEvent)]
    pub fn encode_mouse_event(&self, event: JsValue) -> Result<JsValue, JsValue> {
        let policy = match self.service.active_encoding().as_str() {
            "DEFAULT" => Encoding::Default,
            "SGR" => Encoding::Sgr,
            "SGR_PIXELS" => Encoding::SgrPixels,
            name => {
                return Err(js_sys::Error::new(&format!(
                    "unsupported raw mouse encoding \"{name}\""
                ))
                .into())
            }
        };
        Ok(to_js_string(&policy.encode(&RawMouseEvent(event))?).into())
    }

    pub fn reset(&self) -> Result<(), JsValue> {
        self.service.reset().map_err(|error| to_js_error(&error))
    }

    /// Only the registered protocol emitter is disposed. Modes and mouse
    /// policies remain usable through a retained service, as in the source.
    pub fn dispose(&self) {
        self.service.dispose();
    }
}
