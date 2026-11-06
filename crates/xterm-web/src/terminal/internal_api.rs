//! Live internal service entry points required by unchanged upstream tests.
//! This is not a replacement CoreTerminal or a claim of complete internal ABI.

use super::{
    internal_buffer::InternalBufferServiceView, internal_modes::CoreModeServiceView,
    mouse_api::BrowserMouseState, Terminal,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
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
    input: JsValue,
    browser: Rc<super::internal_browser::BrowserViewSlots>,
    render: Rc<RefCell<JsValue>>,
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
        native: Rc<headless::Terminal>,
        disposed: Rc<Cell<bool>>,
        render: Rc<RefCell<JsValue>>,
        browser: Rc<super::internal_browser::BrowserViewSlots>,
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
            input: super::internal_input::InternalInputHandler::new(native)?.into(),
            browser,
            render,
        })
    }
}

#[wasm_bindgen]
impl CoreServices {
    #[wasm_bindgen(getter, js_name = _inputHandler)]
    pub fn input_handler(&self) -> JsValue {
        self.input.clone()
    }
    #[wasm_bindgen(getter, js_name = _themeService)]
    pub fn theme_service(&self) -> JsValue {
        self.browser.theme.borrow().clone()
    }
    #[wasm_bindgen(getter, js_name = _coreBrowserService)]
    pub fn browser_service(&self) -> JsValue {
        self.browser.browser.borrow().clone()
    }
    #[wasm_bindgen(getter)]
    pub fn buffer(&self) -> Result<JsValue, JsValue> {
        js_sys::Reflect::get(&self.buffers, &JsValue::from_str("buffer"))
    }
    #[wasm_bindgen(getter, js_name = _store)]
    pub fn store(&self) -> JsValue {
        self.store.clone()
    }

    #[wasm_bindgen(getter, js_name = _renderService)]
    pub fn render_service(&self) -> JsValue {
        self.render.borrow().clone()
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
