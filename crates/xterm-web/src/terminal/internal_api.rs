//! Live internal service entry points required by unchanged upstream tests.
//! This is not a replacement CoreTerminal or a claim of complete internal ABI.

use super::{
    internal_buffer::InternalBufferServiceView, internal_modes::CoreModeServiceView,
    mouse_api::BrowserMouseState, Terminal,
};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::headless;
use xterm_wasm_support::public_api::unicode_service::WebUnicodeService;

#[wasm_bindgen]
pub struct CoreServices {
    unicode: JsValue,
    mouse: JsValue,
    core: JsValue,
    buffers: JsValue,
    store: JsValue,
}

/// Read-only inspection of the real core's disposal flag, not a second store.
#[wasm_bindgen]
pub struct CoreDisposalStore {
    disposed: Rc<Cell<bool>>,
}

#[wasm_bindgen]
impl CoreDisposalStore {
    #[wasm_bindgen(getter, js_name = _isDisposed)]
    pub fn private_is_disposed(&self) -> bool {
        self.disposed.get()
    }

    #[wasm_bindgen(getter, js_name = isDisposed)]
    pub fn is_disposed(&self) -> bool {
        self.disposed.get()
    }
}

impl CoreServices {
    pub(super) fn new(
        native: &headless::Terminal,
        disposed: Rc<Cell<bool>>,
    ) -> Result<Self, JsValue> {
        let (unicode, mouse, core, buffers) = native
            .with_state(|state| {
                (
                    Rc::clone(&state.unicode),
                    Rc::clone(&state.mouse),
                    Rc::clone(&state.core),
                    Rc::clone(&state.buffer_service),
                )
            })
            .map_err(|error| crate::js_error::to_js_error(&error))?;
        Ok(Self {
            unicode: WebUnicodeService::new(unicode).into(),
            mouse: BrowserMouseState::new(mouse).into(),
            core: CoreModeServiceView::new(core).into(),
            buffers: InternalBufferServiceView::new(buffers).into(),
            store: CoreDisposalStore { disposed }.into(),
        })
    }
}

#[wasm_bindgen]
impl CoreServices {
    #[wasm_bindgen(getter, js_name = _store)]
    pub fn store(&self) -> JsValue {
        self.store.clone()
    }

    #[wasm_bindgen(getter, js_name = unicodeService)]
    pub fn unicode_service(&self) -> JsValue {
        self.unicode.clone()
    }

    #[wasm_bindgen(getter, js_name = mouseStateService)]
    pub fn mouse_state_service(&self) -> JsValue {
        self.mouse.clone()
    }
    #[wasm_bindgen(getter, js_name = coreService)]
    pub fn core_service(&self) -> JsValue {
        self.core.clone()
    }
    #[wasm_bindgen(getter, js_name = _bufferService)]
    pub fn buffer_service(&self) -> JsValue {
        self.buffers.clone()
    }
}

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(getter, js_name = _core)]
    pub fn core_services(&self) -> JsValue {
        self.shared.internal.clone()
    }
}
