//! Rust ligature addon. The terminal owns registered JavaScript callback handles;
//! disposal removes the current registration just as the source addon does.
use crate::{
    ligature_host as host,
    ligature_state::{self, JoinState},
    local_fonts::LocalFonts,
};
use js_sys::{Array, JsString};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, prelude::*};
use xterm_addons::ligatures::DEFAULT_FALLBACK_LIGATURES;

thread_local! {
    // The source font module caches its catalog for the whole browser realm.
    // Isolate that required shared behavior here, never in the portable engine.
    static FONT_CATALOG: LocalFonts = LocalFonts::default();
}

#[wasm_bindgen]
pub struct LigaturesAddon {
    fallbacks: JsValue,
    features: JsValue,
    terminal: RefCell<Option<JsValue>>,
    joiner_id: RefCell<JsValue>,
}
#[wasm_bindgen]
impl LigaturesAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(options: JsValue) -> Result<LigaturesAddon, JsValue> {
        let defaults = Array::new();
        for pattern in DEFAULT_FALLBACK_LIGATURES {
            defaults.push(&JsValue::from_str(pattern));
        }
        let fallbacks = host::sorted_fallbacks(&options, &defaults)?;
        let features = host::features(&options)?;
        Ok(Self {
            fallbacks,
            features,
            terminal: RefCell::new(None),
            joiner_id: RefCell::new(JsValue::UNDEFINED),
        })
    }
    pub fn activate(&self, terminal: JsValue) -> Result<(), JsValue> {
        if !host::element(&terminal)?.is_truthy() {
            return Err(
                js_sys::Error::new("Cannot activate LigaturesAddon before open is called").into(),
            );
        }
        *self.terminal.borrow_mut() = Some(terminal.clone());
        let term = terminal.clone();
        let patterns = self.fallbacks.clone();
        let state = Rc::new(RefCell::new(JoinState::default()));
        let catalog = FONT_CATALOG.with(Clone::clone);
        let callback = Closure::wrap(Box::new(move |text: JsString| {
            ligature_state::join(&term, &patterns, &state, &catalog, &text)
        })
            as Box<dyn FnMut(JsString) -> Result<Array, JsValue>>)
        .into_js_value();
        let id = host::register(&terminal, &callback)?;
        *self.joiner_id.borrow_mut() = id;
        host::set_features(&terminal, &self.features)
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        let id = self.joiner_id.borrow().clone();
        if !id.is_undefined() {
            let terminal = self.terminal.borrow().clone().unwrap_or(JsValue::UNDEFINED);
            host::deregister(&terminal, &id)?;
            *self.joiner_id.borrow_mut() = JsValue::UNDEFINED;
        }
        let terminal = self.terminal.borrow().clone().unwrap_or(JsValue::UNDEFINED);
        host::clear_features(&terminal)
    }
}
