//! WebLinksAddon exports use the terminal's pre-open provider registry.
use super::{
    matcher::{BrowserLinkMatcher, ObjectLinkMatcher},
    provider::BrowserWebLinkTerminal,
};
use crate::{terminal::Terminal, utf16::to_js_string};
use js_sys::{Array, Function, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::MouseEvent;
use xterm_addons::{
    link_snapshot::LinkSnapshot,
    web_link_provider::{Activate, LinkOptions, WebLinksAddon as Inner},
    web_links::{LinkError, LinkMatcher, LinkRange},
};
use xterm_core::lifecycle::Disposable;

fn error(value: JsValue) -> LinkError {
    LinkError::host(crate::js_error::JsCallbackError(value))
}
fn nullish(value: &JsValue) -> bool {
    value.is_null() || value.is_undefined()
}
fn call(value: &JsValue, args: &Array) -> Result<(), LinkError> {
    // Reflect.apply produces the browser's TypeError for a non-callable value.
    let apply = Reflect::get(&js_sys::global(), &"Reflect".into()).map_err(error)?;
    let method = Reflect::get(&apply, &"apply".into())
        .map_err(error)?
        .unchecked_into::<Function>();
    method
        .call3(&apply, value, &JsValue::UNDEFINED, args)
        .map_err(error)?;
    Ok(())
}
fn args(event: &MouseEvent, uri: &[u16]) -> Array {
    let args = Array::new();
    args.push(event.as_ref());
    args.push(&to_js_string(uri));
    args
}
fn range(range: LinkRange) -> Result<JsValue, LinkError> {
    let result = Object::new();
    for (key, position) in [("start", range.start), ("end", range.end)] {
        let point = Object::new();
        Reflect::set(&point, &"x".into(), &JsValue::from_f64(position.x as f64)).map_err(error)?;
        Reflect::set(&point, &"y".into(), &JsValue::from_f64(position.y as f64)).map_err(error)?;
        Reflect::set(&result, &key.into(), &point).map_err(error)?;
    }
    Ok(result.into())
}
fn callback(value: JsValue) -> Option<Activate<MouseEvent>> {
    if value.is_undefined() {
        None
    } else {
        Some(Rc::new(move |event, uri| call(&value, &args(event, uri))))
    }
}

#[wasm_bindgen(js_name=WebLinksAddon)]
pub struct WebLinksAddon {
    addon: Inner<MouseEvent>,
    terminals: RefCell<Vec<Rc<BrowserWebLinkTerminal>>>,
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
        let hover = options.clone();
        let leave = options.clone();
        let pattern = options.clone();
        let native_options = LinkOptions {
            matcher_factory: Some(Rc::new(move || {
                let regex = super::js_property::get(&pattern, "urlRegex").map_err(error)?;
                Ok(if nullish(&regex) {
                    Rc::new(BrowserLinkMatcher) as Rc<dyn LinkMatcher>
                } else {
                    Rc::new(ObjectLinkMatcher::new(regex)) as Rc<dyn LinkMatcher>
                })
            })),
            hover_factory: Some(Rc::new(move |location| {
                let location = range(location)?;
                let hover = hover.clone();
                Ok(Rc::new(move |event: &MouseEvent, uri: &[u16]| {
                    if super::js_property::get(&hover, "hover")
                        .map_err(error)?
                        .is_truthy()
                    {
                        // Source reads the option twice, so accessor side effects
                        // between the truthiness check and invocation are visible.
                        let callback = super::js_property::get(&hover, "hover").map_err(error)?;
                        let args = args(event, uri);
                        args.push(&location);
                        call(&callback, &args)?;
                    }
                    Ok(())
                }) as Activate<MouseEvent>)
            })),
            leave_factory: Some(Rc::new(move || {
                let value = super::js_property::get(&leave, "leave").map_err(error)?;
                Ok(if value.is_truthy() {
                    callback(value)
                } else {
                    None
                })
            })),
            ..LinkOptions::default()
        };
        Self {
            addon: Inner::new(
                Rc::new(BrowserLinkMatcher),
                callback(handler),
                native_options,
            ),
            terminals: RefCell::default(),
        }
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        let native = Rc::downgrade(&terminal.shared_native());
        let host = Rc::new(BrowserWebLinkTerminal {
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
        });
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
