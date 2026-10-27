//! A callable DOM host for component integration tests. This intentionally is
//! not named Terminal: the engine and browser services must be connected first.

use crate::dom::{BrowserDom, DomOptions};
use wasm_bindgen::prelude::*;
use web_sys::{FocusOptions, HtmlElement, HtmlTextAreaElement};

#[wasm_bindgen]
pub struct BrowserHost {
    dom: Option<BrowserDom>,
    disposed: bool,
}

#[wasm_bindgen]
impl BrowserHost {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            dom: None,
            disposed: false,
        }
    }

    /// Reopening does not recreate nodes or move the original terminal root.
    pub fn open(&mut self, parent: HtmlElement) -> Result<(), JsValue> {
        self.check_live()?;
        if self.dom.is_none() {
            self.dom = Some(BrowserDom::new(&parent, &DomOptions::default())?);
        }
        Ok(())
    }

    #[wasm_bindgen(getter)]
    pub fn element(&self) -> Option<HtmlElement> {
        self.dom.as_ref().map(|dom| dom.element.clone())
    }

    #[wasm_bindgen(getter, js_name = screenElement)]
    pub fn screen_element(&self) -> Option<HtmlElement> {
        self.dom.as_ref().map(|dom| dom.screen.clone())
    }

    #[wasm_bindgen(getter)]
    pub fn textarea(&self) -> Option<HtmlTextAreaElement> {
        self.dom.as_ref().map(|dom| dom.textarea.clone())
    }

    pub fn focus(&self) -> Result<(), JsValue> {
        self.check_live()?;
        if let Some(dom) = &self.dom {
            let options = FocusOptions::new();
            options.set_prevent_scroll(true);
            dom.textarea.focus_with_options(&options)?;
        }
        Ok(())
    }

    pub fn blur(&self) -> Result<(), JsValue> {
        self.check_live()?;
        if let Some(dom) = &self.dom {
            dom.textarea.blur()?;
        }
        Ok(())
    }

    pub fn dispose(&mut self) {
        self.dom.take();
        self.disposed = true;
    }
}

impl Default for BrowserHost {
    fn default() -> Self {
        Self::new()
    }
}

impl BrowserHost {
    fn check_live(&self) -> Result<(), JsValue> {
        if self.disposed {
            Err(js_sys::Error::new("Browser host has been disposed").into())
        } else {
            Ok(())
        }
    }
}
