//! Read-only browser service views retain the real service, not a second state.
use crate::{
    core_browser::CoreBrowserService,
    theme::{ColorRef, SharedColors, ThemeService},
};
use js_sys::Array;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

pub(super) struct BrowserViewSlots {
    pub theme: RefCell<JsValue>,
    pub browser: RefCell<JsValue>,
}
impl Default for BrowserViewSlots {
    fn default() -> Self {
        Self {
            theme: RefCell::new(JsValue::UNDEFINED),
            browser: RefCell::new(JsValue::UNDEFINED),
        }
    }
}
impl BrowserViewSlots {
    pub fn install(&self, theme: Rc<ThemeService>, browser: Rc<CoreBrowserService>) {
        let colors: JsValue = InternalColors {
            colors: theme.colors(),
        }
        .into();
        *self.theme.borrow_mut() = InternalTheme { colors }.into();
        *self.browser.borrow_mut() = InternalCoreBrowser { service: browser }.into();
    }
}
#[wasm_bindgen]
pub struct InternalTheme {
    colors: JsValue,
}
#[wasm_bindgen]
impl InternalTheme {
    #[wasm_bindgen(getter)]
    pub fn colors(&self) -> JsValue {
        self.colors.clone()
    }
}
#[wasm_bindgen]
pub struct InternalColors {
    colors: SharedColors,
}
#[wasm_bindgen]
impl InternalColors {
    #[wasm_bindgen(getter)]
    pub fn foreground(&self) -> InternalColor {
        InternalColor(self.colors.borrow().foreground.clone())
    }
    #[wasm_bindgen(getter)]
    pub fn background(&self) -> InternalColor {
        InternalColor(self.colors.borrow().background.clone())
    }
    #[wasm_bindgen(getter)]
    pub fn cursor(&self) -> InternalColor {
        InternalColor(self.colors.borrow().cursor.clone())
    }
    #[wasm_bindgen(getter, js_name = cursorAccent)]
    pub fn cursor_accent(&self) -> InternalColor {
        InternalColor(self.colors.borrow().cursor_accent.clone())
    }
    #[wasm_bindgen(getter)]
    pub fn ansi(&self) -> Array {
        self.colors
            .borrow()
            .ansi
            .iter()
            .cloned()
            .map(|color| JsValue::from(InternalColor(color)))
            .collect()
    }
}
#[wasm_bindgen]
pub struct InternalColor(ColorRef);
#[wasm_bindgen]
impl InternalColor {
    #[wasm_bindgen(getter)]
    pub fn css(&self) -> js_sys::JsString {
        crate::utf16::to_js_string(&self.0.css_units())
    }
    #[wasm_bindgen(getter)]
    pub fn rgba(&self) -> u32 {
        self.0.borrow().rgba
    }
}
#[wasm_bindgen]
pub struct InternalCoreBrowser {
    service: Rc<CoreBrowserService>,
}
#[wasm_bindgen]
impl InternalCoreBrowser {
    #[wasm_bindgen(getter)]
    pub fn window(&self) -> web_sys::Window {
        self.service.window()
    }
    #[wasm_bindgen(getter, js_name = mainDocument)]
    pub fn document(&self) -> web_sys::Document {
        self.service.main_document()
    }
    #[wasm_bindgen(getter)]
    pub fn dpr(&self) -> Result<f64, JsValue> {
        self.service.dpr()
    }
    #[wasm_bindgen(getter, js_name = isFocused)]
    pub fn focused(&self) -> Result<bool, JsValue> {
        self.service.is_focused()
    }
}
