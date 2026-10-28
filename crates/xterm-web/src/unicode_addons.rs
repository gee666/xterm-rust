//! Rust-backed Unicode addon constructors and lifecycle bindings.
//! The terminal's native Unicode service owns all width and joining behavior.
use crate::{js_error::to_js_error, terminal::Terminal};
use wasm_bindgen::prelude::*;
use xterm_core::unicode_addons::{
    Unicode11Addon as NativeUnicode11Addon, UnicodeGraphemesAddon as NativeUnicodeGraphemesAddon,
};

#[derive(Default)]
#[wasm_bindgen]
pub struct Unicode11Addon {
    inner: NativeUnicode11Addon,
}

#[wasm_bindgen]
impl Unicode11Addon {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let native = terminal.shared_native();
        let unicode = native.unicode().map_err(|error| to_js_error(&error))?;
        self.inner
            .activate(unicode.as_ref())
            .map_err(|error| to_js_error(&error))
    }

    /// Source disposal leaves the registered provider available for selection.
    pub fn dispose(&self) {
        self.inner.dispose();
    }
}

#[derive(Default)]
#[wasm_bindgen]
pub struct UnicodeGraphemesAddon {
    inner: NativeUnicodeGraphemesAddon,
}

#[wasm_bindgen]
impl UnicodeGraphemesAddon {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let native = terminal.shared_native();
        let unicode = native.unicode().map_err(|error| to_js_error(&error))?;
        self.inner
            .activate(unicode)
            .map_err(|error| to_js_error(&error))
    }

    pub fn dispose(&self) -> Result<(), JsValue> {
        self.inner.dispose().map_err(|error| to_js_error(&error))
    }
}
