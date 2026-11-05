//! WebLinksAddon exports use the terminal's pre-open provider registry.
use super::{
    javascript_provider::{handler_or_default, JavaScriptWebLinkTerminal},
    matcher::{BrowserLinkMatcher, ObjectLinkMatcher},
    provider::BrowserWebLinkTerminal,
};
use crate::terminal::Terminal;
use js_sys::Object;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use web_sys::MouseEvent;
use xterm_addons::{
    link_snapshot::LinkSnapshot,
    web_link_provider::{LinkOptions, WebLinksAddon as Inner},
    web_links::{LinkError, LinkMatcher},
};
use xterm_core::lifecycle::Disposable;

fn error(value: JsValue) -> LinkError {
    LinkError::host(crate::js_error::JsCallbackError(value))
}

#[wasm_bindgen(js_name=WebLinksAddon)]
pub struct WebLinksAddon {
    addon: Inner<MouseEvent>,
    handler: JsValue,
    options: JsValue,
    terminals: RefCell<Vec<Rc<JavaScriptWebLinkTerminal>>>,
}
#[wasm_bindgen]
impl WebLinksAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(handler: JsValue, options: JsValue) -> WebLinksAddon {
        let options = if options.is_undefined() {
            Object::new().into()
        } else {
            options
        };
        let pattern = options.clone();
        let native_options = LinkOptions {
            matcher_factory: Some(Rc::new(move || {
                let regex = super::js_property::get(&pattern, "urlRegex").map_err(error)?;
                Ok(if regex.is_null() || regex.is_undefined() {
                    Rc::new(BrowserLinkMatcher) as Rc<dyn LinkMatcher>
                } else {
                    Rc::new(ObjectLinkMatcher::new(regex)) as Rc<dyn LinkMatcher>
                })
            })),
            // JS binding objects attach callbacks after all Rust URL discovery.
            // No option reads or detached callback range are created here.
            ..LinkOptions::default()
        };
        Self {
            addon: Inner::new(Rc::new(BrowserLinkMatcher), None, native_options),
            handler: handler_or_default(handler),
            options,
            terminals: RefCell::default(),
        }
    }
    /// The unchanged upstream suite updates this live constructor options bag.
    #[wasm_bindgen(getter,js_name=_options)]
    pub fn options_compat(&self) -> JsValue {
        self.options.clone()
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let native = Rc::downgrade(&terminal.shared_native());
        let host = Rc::new(JavaScriptWebLinkTerminal::new(
            BrowserWebLinkTerminal {
                providers: terminal.shared.link_providers.clone(),
                snapshot: Rc::new(move || {
                    let native = native
                        .upgrade()
                        .ok_or_else(|| LinkError::Message("link terminal was dropped".into()))?;
                    let snapshot = native
                        .with_state(LinkSnapshot::from_state)
                        .map_err(LinkError::host)??;
                    Ok(Box::new(snapshot))
                }),
            },
            self.handler.clone(),
            self.options.clone(),
        ));
        self.addon
            .activate(host.clone())
            .map_err(|e| crate::js_error::to_js_error(&e))?;
        self.terminals.borrow_mut().push(host);
        Ok(())
    }
    pub fn dispose(&self) {
        self.addon.dispose();
    }
}
