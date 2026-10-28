// Copyright (c) 2024 The xterm.js authors. All rights reserved. MIT license.
//! Web-font addon decisions execute in Rust. The terminal argument can be the
//! Rust/WASM terminal or a test host with the same live options object.

use crate::font_set::{font_property, load_fonts, load_selected, registered_faces, wait_ready};
use crate::utf16::{from_js_string, to_js_string};
use js_sys::{Array, JsString, Promise, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::future_to_promise;
use web_sys::Document;
use xterm_addons::web_fonts::relayout_plan;

type TerminalSlot = Rc<RefCell<Option<JsValue>>>;

fn document() -> Result<Document, JsValue> {
    web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| js_sys::Error::new("WebFontsAddon requires a document").into())
}

fn terminal_options(slot: &TerminalSlot) -> Result<JsValue, JsValue> {
    // Never retain a RefCell borrow across JS getters or setters.
    let terminal = slot.borrow().clone().ok_or_else(|| {
        js_sys::TypeError::new("Cannot read properties of undefined (reading 'options')")
    })?;
    Reflect::get(&terminal, &JsValue::from_str("options"))
}

fn set_family(slot: &TerminalSlot, family: &[u16]) -> Result<(), JsValue> {
    let options = terminal_options(slot)?;
    let success = Reflect::set(
        &options,
        &JsValue::from_str("fontFamily"),
        &to_js_string(family),
    )?;
    if success {
        Ok(())
    } else {
        Err(js_sys::TypeError::new("Cannot assign to fontFamily").into())
    }
}

async fn relayout(slot: TerminalSlot, document: Document) -> Result<JsValue, JsValue> {
    if slot.borrow().is_none() {
        return Ok(JsValue::UNDEFINED);
    }
    wait_ready(&document).await?;
    let options = terminal_options(&slot)?;
    let family = Reflect::get(&options, &JsValue::from_str("fontFamily"))?;
    let family = if family.is_undefined() || family.is_null() {
        Vec::new()
    } else {
        from_js_string(
            &family
                .dyn_into::<JsString>()
                .map_err(|_| js_sys::TypeError::new("fontFamily must be a string"))?,
        )
    };
    let registered: Vec<_> = registered_faces(&document.fonts())?
        .iter()
        .map(|face| font_property(face, "family"))
        .collect::<Result<_, _>>()?;
    let Some(plan) = relayout_plan(&family, &registered) else {
        return Ok(JsValue::UNDEFINED);
    };
    let requests = Array::new();
    for name in plan.load_families {
        requests.push(&to_js_string(&name));
    }
    load_selected(&document, Some(&requests)).await?;
    if slot.borrow().is_some() {
        set_family(&slot, &plan.temporary_family)?;
        set_family(&slot, &plan.restore_family)?;
    }
    Ok(JsValue::UNDEFINED)
}

#[wasm_bindgen(js_name = loadFonts)]
pub fn load_web_fonts(fonts: Option<Array>) -> Promise {
    future_to_promise(async move { Ok(load_fonts(&document()?, fonts.as_ref()).await?.into()) })
}

#[wasm_bindgen]
pub struct WebFontsAddon {
    terminal: TerminalSlot,
    initial_relayout: bool,
}

#[wasm_bindgen]
impl WebFontsAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(initial_relayout: Option<bool>) -> Self {
        Self {
            terminal: Rc::new(RefCell::new(None)),
            initial_relayout: initial_relayout.unwrap_or(true),
        }
    }

    #[wasm_bindgen(getter, js_name = initialRelayout)]
    pub fn initial_relayout(&self) -> bool {
        self.initial_relayout
    }

    #[wasm_bindgen(setter, js_name = initialRelayout)]
    pub fn set_initial_relayout(&mut self, value: bool) {
        self.initial_relayout = value;
    }

    pub fn activate(&self, terminal: JsValue) -> Result<(), JsValue> {
        *self.terminal.borrow_mut() = Some(terminal);
        if self.initial_relayout {
            let document = document()?;
            let slot = Rc::clone(&self.terminal);
            // As upstream's ready.then callback, rejection remains observable
            // as an unhandled promise rejection if the host does not observe it.
            drop(future_to_promise(async move {
                wait_ready(&document).await?;
                relayout(slot, document).await
            }));
        }
        Ok(())
    }

    pub fn dispose(&self) {
        self.terminal.borrow_mut().take();
    }

    #[wasm_bindgen(js_name = loadFonts)]
    pub fn load_fonts(&self, fonts: Option<Array>) -> Promise {
        load_web_fonts(fonts)
    }

    pub fn relayout(&self) -> Promise {
        let slot = Rc::clone(&self.terminal);
        future_to_promise(async move {
            if slot.borrow().is_none() {
                return Ok(JsValue::UNDEFINED);
            }
            relayout(slot, document()?).await
        })
    }
}
