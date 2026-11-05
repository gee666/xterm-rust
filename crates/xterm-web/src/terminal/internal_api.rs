//! Live internal service entry points required by unchanged upstream tests.
//! This is not a replacement CoreTerminal or a claim of complete internal ABI.

use super::{mouse_api::BrowserMouseState, Terminal};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use xterm_core::{mouse_state::MouseStateService, unicode_service::UnicodeService};
use xterm_wasm_support::public_api::unicode_service::WebUnicodeService;

#[wasm_bindgen]
pub struct CoreServices {
    unicode: JsValue,
    mouse: JsValue,
}

impl CoreServices {
    pub(super) fn new(unicode: Rc<UnicodeService>, mouse: Rc<MouseStateService>) -> Self {
        Self {
            unicode: WebUnicodeService::new(unicode).into(),
            mouse: BrowserMouseState::new(mouse).into(),
        }
    }
}

#[wasm_bindgen]
impl CoreServices {
    #[wasm_bindgen(getter, js_name = unicodeService)]
    pub fn unicode_service(&self) -> JsValue {
        self.unicode.clone()
    }

    #[wasm_bindgen(getter, js_name = mouseStateService)]
    pub fn mouse_state_service(&self) -> JsValue {
        self.mouse.clone()
    }
}

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(getter, js_name = _core)]
    pub fn core_services(&self) -> JsValue {
        self.shared.internal.clone()
    }
}
