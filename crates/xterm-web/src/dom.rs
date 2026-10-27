// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Adapted from browser/CoreBrowserTerminal.ts.

//! DOM ownership only. Parsing, input dispatch and rendering are separate owners.

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement, HtmlTextAreaElement};

#[derive(Clone)]
pub struct DomOptions {
    pub allow_transparency: bool,
    pub disable_stdin: bool,
    pub is_chrome_os: bool,
    pub prompt_label: String,
    pub document_override: Option<Document>,
}

impl Default for DomOptions {
    fn default() -> Self {
        Self {
            allow_transparency: false,
            disable_stdin: false,
            is_chrome_os: false,
            prompt_label: "Terminal input".into(),
            document_override: None,
        }
    }
}

/// The nodes remain valid after disposal but the root is detached. The terminal
/// coordinator must dispose component listeners before dropping this owner.
pub struct BrowserDom {
    pub document: Document,
    pub element: HtmlElement,
    pub viewport: HtmlElement,
    pub screen: HtmlElement,
    pub helpers: HtmlElement,
    pub textarea: HtmlTextAreaElement,
    pub composition: HtmlElement,
    disposed: bool,
}

impl BrowserDom {
    pub fn new(parent: &HtmlElement, options: &DomOptions) -> Result<Self, JsValue> {
        let document = options
            .document_override
            .clone()
            .or_else(|| parent.owner_document())
            .ok_or_else(|| js_sys::Error::new("Terminal requires a parent document."))?;
        let element = div(&document, "terminal xterm")?;
        element.set_dir("ltr");
        element
            .class_list()
            .toggle_with_force("allow-transparency", options.allow_transparency)?;

        let viewport = div(&document, "xterm-viewport")?;
        let screen = div(&document, "xterm-screen")?;
        let helpers = div(&document, "xterm-helpers")?;
        let textarea: HtmlTextAreaElement = document.create_element("textarea")?.dyn_into()?;
        textarea.set_class_name("xterm-helper-textarea");
        textarea.set_attribute("aria-label", &options.prompt_label)?;
        if !options.is_chrome_os {
            textarea.set_attribute("aria-multiline", "false")?;
        }
        for (key, value) in [
            ("autocomplete", "off"),
            ("autocorrect", "off"),
            ("autocapitalize", "off"),
            ("spellcheck", "false"),
        ] {
            textarea.set_attribute(key, value)?;
        }
        textarea.set_tab_index(0);
        textarea.set_read_only(options.disable_stdin);
        let composition = div(&document, "composition-view")?;
        helpers.append_child(&textarea)?;
        helpers.append_child(&composition)?;
        screen.append_child(&helpers)?;

        // Build detached so a failed DOM operation cannot leave a partial root
        // attached to the user's parent.
        let fragment = document.create_document_fragment();
        fragment.append_child(&viewport)?;
        fragment.append_child(&screen)?;
        element.append_child(&fragment)?;
        parent.append_child(&element)?;
        Ok(Self {
            document,
            element,
            viewport,
            screen,
            helpers,
            textarea,
            composition,
            disposed: false,
        })
    }

    pub fn is_disposed(&self) -> bool {
        self.disposed
    }

    pub fn set_allow_transparency(&self, enabled: bool) -> Result<(), JsValue> {
        self.element
            .class_list()
            .toggle_with_force("allow-transparency", enabled)?;
        Ok(())
    }

    pub fn set_disable_stdin(&self, disabled: bool) {
        self.textarea.set_read_only(disabled);
    }

    pub fn dispose(&mut self) {
        if !self.disposed {
            self.element.remove();
            self.disposed = true;
        }
    }
}

impl Drop for BrowserDom {
    fn drop(&mut self) {
        self.dispose();
    }
}

fn div(document: &Document, class_name: &str) -> Result<HtmlElement, JsValue> {
    let element: HtmlElement = document.create_element("div")?.dyn_into()?;
    element.set_class_name(class_name);
    Ok(element)
}
