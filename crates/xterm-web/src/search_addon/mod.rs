// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Callable search bindings over the Rust engine and shared browser services.
mod host;
mod options;
mod timer;
use host::BrowserSearchHost;
use js_sys::{Function, Object, Reflect};
use std::{error::Error, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use xterm_addons::search::{SearchAddon as NativeAddon, SearchOptionsHandle};
use xterm_core::{event::Event, lifecycle::DisposableHandle};

#[wasm_bindgen]
pub struct SearchSubscription {
    handle: DisposableHandle,
}
#[wasm_bindgen]
impl SearchSubscription {
    pub fn dispose(&self) {
        self.handle.dispose();
    }
}

type EventFunction = Closure<dyn FnMut(Function) -> Result<JsValue, JsValue>>;
fn event<T: 'static>(
    event: Event<T>,
    convert: impl Fn(&T) -> Result<JsValue, JsValue> + 'static,
) -> EventFunction {
    let convert = Rc::new(convert);
    Closure::new(move |listener: Function| {
        let convert = Rc::clone(&convert);
        let handle = event.try_subscribe(move |value| {
            let value = convert(value).map_err(crate::js_error::callback_error)?;
            listener
                .call1(&JsValue::UNDEFINED, &value)
                .map_err(crate::js_error::callback_error)?;
            Ok(())
        });
        Ok(SearchSubscription { handle }.into())
    })
}
fn function(event: &EventFunction) -> Function {
    event.as_ref().unchecked_ref::<Function>().clone()
}

#[wasm_bindgen]
pub struct SearchAddon {
    addon: NativeAddon,
    before: EventFunction,
    after: EventFunction,
    changed: EventFunction,
}
#[wasm_bindgen]
impl SearchAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(initial_options: JsValue) -> Result<SearchAddon, JsValue> {
        let limit = options::property(&initial_options, "highlightLimit")?;
        let addon = if limit.is_null() || limit.is_undefined() {
            NativeAddon::default()
        } else {
            NativeAddon::with_limit(Rc::new(options::JsLimit(limit)))
        };
        let before = event(addon.on_before_search(), |_| Ok(JsValue::UNDEFINED));
        let after = event(addon.on_after_search(), |_| Ok(JsValue::UNDEFINED));
        let changed = event(addon.on_did_change_results(), |result| {
            let object = Object::new();
            Reflect::set(
                &object,
                &"resultIndex".into(),
                &(result.result_index as f64).into(),
            )?;
            Reflect::set(
                &object,
                &"resultCount".into(),
                &(result.result_count as f64).into(),
            )?;
            Ok(object.into())
        });
        Ok(Self {
            addon,
            before,
            after,
            changed,
        })
    }
    pub fn activate(&self, terminal: &crate::terminal::Terminal) -> Result<(), JsValue> {
        self.addon
            .activate(Rc::new(BrowserSearchHost::new(terminal)?))
            .map_err(|cause| error(&cause))
    }
    #[wasm_bindgen(js_name = findNext)]
    pub fn find_next(&self, term: JsValue, options: JsValue) -> Result<bool, JsValue> {
        self.addon
            .find_next(&options::term(term)?, request(options))
            .map_err(|cause| error(&cause))
    }
    #[wasm_bindgen(js_name = findPrevious)]
    pub fn find_previous(&self, term: JsValue, options: JsValue) -> Result<bool, JsValue> {
        self.addon
            .find_previous(&options::term(term)?, request(options))
            .map_err(|cause| error(&cause))
    }
    #[wasm_bindgen(js_name = clearDecorations)]
    pub fn clear_decorations(&self) -> Result<(), JsValue> {
        self.addon
            .clear_decorations(false)
            .map_err(|cause| error(&cause))
    }
    #[wasm_bindgen(js_name = clearActiveDecoration)]
    pub fn clear_active_decoration(&self) -> Result<(), JsValue> {
        self.addon
            .clear_active_decoration()
            .map_err(|cause| error(&cause))
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.addon.dispose().map_err(|cause| error(&cause))
    }
    #[wasm_bindgen(getter, js_name = onBeforeSearch)]
    pub fn on_before_search(&self) -> Function {
        function(&self.before)
    }
    #[wasm_bindgen(getter, js_name = onAfterSearch)]
    pub fn on_after_search(&self) -> Function {
        function(&self.after)
    }
    #[wasm_bindgen(getter, js_name = onDidChangeResults)]
    pub fn on_did_change_results(&self) -> Function {
        function(&self.changed)
    }
}
pub(super) fn error(error: &xterm_addons::search::SearchAddonError) -> JsValue {
    let mut cause: Option<&(dyn Error + 'static)> = Some(error);
    while let Some(current) = cause {
        if let Some(error) = current.downcast_ref::<xterm_core::search::SearchError>() {
            match error {
                xterm_core::search::SearchError::WrappedLineRecursion
                | xterm_core::search::SearchError::IndexOverflow => {
                    return js_sys::RangeError::new(&error.to_string()).into()
                }
                xterm_core::search::SearchError::InvalidCellData => {
                    return js_sys::TypeError::new(&error.to_string()).into()
                }
                xterm_core::search::SearchError::InvalidRegex(_) => {
                    return js_sys::SyntaxError::new(&error.to_string()).into()
                }
                _ => {}
            }
        }
        cause = current.source();
    }
    crate::js_error::to_js_error(error)
}
fn request(options: JsValue) -> Option<SearchOptionsHandle> {
    (!options.is_undefined() && !options.is_null())
        .then(|| Rc::new(options::JsOptions(options)) as SearchOptionsHandle)
}
