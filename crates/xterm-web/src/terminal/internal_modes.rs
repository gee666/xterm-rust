//! Read-only internal DEC mode observers over the actual Rust CoreService.
//! Reset replaces the source object; retained older views freeze at that boundary.
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::core_service::{CoreService, CursorStyle, OwnedPrivateModesCapture};

struct CachedModes {
    capture: Rc<OwnedPrivateModesCapture>,
    object: JsValue,
}
#[wasm_bindgen]
pub struct CoreModeServiceView {
    core: Rc<CoreService>,
    modes: RefCell<Option<CachedModes>>,
}
impl CoreModeServiceView {
    pub(super) fn new(core: Rc<CoreService>) -> Self {
        Self {
            core,
            modes: RefCell::new(None),
        }
    }
}
#[wasm_bindgen]
impl CoreModeServiceView {
    #[wasm_bindgen(getter, js_name = decPrivateModes)]
    pub fn dec_private_modes(&self) -> JsValue {
        if let Some(cached) = self.modes.borrow().as_ref() {
            if cached.capture.is_current() {
                return cached.object.clone();
            }
        }
        let capture = Rc::new(self.core.capture_private_modes_owned());
        let object: JsValue = DecPrivateModesView {
            capture: Rc::clone(&capture),
        }
        .into();
        *self.modes.borrow_mut() = Some(CachedModes {
            capture,
            object: object.clone(),
        });
        object
    }
}
#[wasm_bindgen]
pub struct DecPrivateModesView {
    capture: Rc<OwnedPrivateModesCapture>,
}
#[wasm_bindgen]
impl DecPrivateModesView {
    #[wasm_bindgen(getter, js_name = applicationCursorKeys)]
    pub fn application_cursor_keys(&self) -> bool {
        self.capture.read().application_cursor_keys
    }
    #[wasm_bindgen(getter, js_name = applicationKeypad)]
    pub fn application_keypad(&self) -> bool {
        self.capture.read().application_keypad
    }
    #[wasm_bindgen(getter, js_name = bracketedPasteMode)]
    pub fn bracketed_paste_mode(&self) -> bool {
        self.capture.read().bracketed_paste_mode
    }
    #[wasm_bindgen(getter, js_name = colorSchemeUpdates)]
    pub fn color_scheme_updates(&self) -> bool {
        self.capture.read().color_scheme_updates
    }
    #[wasm_bindgen(getter, js_name = cursorBlink)]
    pub fn cursor_blink(&self) -> JsValue {
        self.capture
            .read()
            .cursor_blink
            .map(JsValue::from_bool)
            .unwrap_or(JsValue::UNDEFINED)
    }
    #[wasm_bindgen(getter, js_name = cursorStyle)]
    pub fn cursor_style(&self) -> JsValue {
        match self.capture.read().cursor_style {
            Some(CursorStyle::Block) => JsValue::from_str("block"),
            Some(CursorStyle::Underline) => JsValue::from_str("underline"),
            Some(CursorStyle::Bar) => JsValue::from_str("bar"),
            None => JsValue::UNDEFINED,
        }
    }
    #[wasm_bindgen(getter)]
    pub fn origin(&self) -> bool {
        self.capture.read().origin
    }
    #[wasm_bindgen(getter, js_name = reverseWraparound)]
    pub fn reverse_wraparound(&self) -> bool {
        self.capture.read().reverse_wraparound
    }
    #[wasm_bindgen(getter, js_name = sendFocus)]
    pub fn send_focus(&self) -> bool {
        self.capture.read().send_focus
    }
    #[wasm_bindgen(getter, js_name = synchronizedOutput)]
    pub fn synchronized_output(&self) -> bool {
        self.capture.read().synchronized_output
    }
    #[wasm_bindgen(getter, js_name = win32InputMode)]
    pub fn win32_input_mode(&self) -> bool {
        self.capture.read().win32_input_mode
    }
    #[wasm_bindgen(getter)]
    pub fn wraparound(&self) -> bool {
        self.capture.read().wraparound
    }
}
