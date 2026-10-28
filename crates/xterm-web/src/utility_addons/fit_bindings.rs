//! FitAddon binding reads actual renderer measurements and live native options.
use super::fit::{BrowserFitTerminal, ScrollbarOptions};
use crate::terminal::{Shared, Terminal};
use js_sys::{Object, Reflect};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::prelude::*;
use xterm_addons::fit::{FitAddon as Inner, FitError};
use xterm_core::options::OptionValue;

#[wasm_bindgen(js_name=FitAddon)]
#[derive(Default)]
pub struct FitAddon {
    addon: RefCell<Inner>,
    terminal: RefCell<Option<Rc<BrowserFitTerminal>>>,
}
fn shared(weak: &Weak<Shared>) -> Result<Rc<Shared>, FitError> {
    weak.upgrade()
        .ok_or_else(|| FitError::Message("fit terminal was dropped".into()))
}
fn nullish(value: &OptionValue) -> bool {
    matches!(value, OptionValue::Null | OptionValue::Undefined)
}
fn scrollbar(weak: &Weak<Shared>) -> Result<ScrollbarOptions, FitError> {
    let shared = shared(weak)?;
    let options = shared.options.object();
    let get = |key: &str| -> Result<OptionValue, FitError> {
        let value = Reflect::get(&options, &key.into())
            .map_err(|error| FitError::host(crate::js_error::JsCallbackError(error)))?;
        crate::terminal::options::from_js(&value)
            .map_err(|error| FitError::host(crate::js_error::JsCallbackError(error)))
    };
    // The public descriptors can be redefined. Read in source order and do not
    // evaluate the width getter when scrollback or scrollbar visibility wins.
    let value = get("scrollbar")?;
    let show = if nullish(&value) {
        None
    } else {
        let show = value
            .get_property("showScrollbar")
            .map_err(FitError::host)?;
        if nullish(&show) {
            None
        } else {
            Some(show.is_truthy())
        }
    };
    let scrollback = match get("scrollback")? {
        OptionValue::Number(n) => n,
        _ => f64::NAN,
    };
    let width = if scrollback == 0.0 || show == Some(false) {
        None
    } else {
        let value = get("scrollbar")?;
        if nullish(&value) {
            None
        } else {
            let width = value.get_property("width").map_err(FitError::host)?;
            if nullish(&width) {
                None
            } else {
                Some(width.to_number().map_err(FitError::host)?)
            }
        }
    };
    Ok(ScrollbarOptions {
        scrollback,
        show,
        width,
    })
}
#[wasm_bindgen]
impl FitAddon {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self::default()
    }
    pub fn activate(&self, terminal: &Terminal) {
        let element = Rc::downgrade(&terminal.shared);
        let dimensions = element.clone();
        let options = element.clone();
        let resize = element.clone();
        let host = Rc::new(BrowserFitTerminal {
            element: Rc::new(move || {
                element
                    .upgrade()
                    .and_then(|shared| shared.dom.borrow().as_ref().map(|dom| dom.element.clone()))
            }),
            dimensions: Rc::new(move || {
                dimensions
                    .upgrade()
                    .and_then(|shared| Terminal { shared }.render_dimensions())
            }),
            scrollbar: Rc::new(move || scrollbar(&options)),
            resize: Rc::new(move |dimensions| {
                let terminal = Terminal {
                    shared: shared(&resize)?,
                };
                terminal
                    .resize(dimensions.cols, dimensions.rows)
                    .map_err(|error| FitError::host(crate::js_error::JsCallbackError(error)))
            }),
        });
        self.addon.borrow_mut().activate(host.clone());
        *self.terminal.borrow_mut() = Some(host);
    }
    #[wasm_bindgen(js_name=proposeDimensions)]
    pub fn propose_dimensions(&self) -> Result<JsValue, JsValue> {
        let addon = self.addon.borrow().clone();
        let Some(dimensions) = addon
            .propose_dimensions()
            .map_err(|error| crate::js_error::to_js_error(&error))?
        else {
            return Ok(JsValue::UNDEFINED);
        };
        let value = Object::new();
        if !Reflect::set(&value, &"cols".into(), &JsValue::from_f64(dimensions.cols))?
            || !Reflect::set(&value, &"rows".into(), &JsValue::from_f64(dimensions.rows))?
        {
            return Err(js_sys::TypeError::new("Fit dimensions assignment rejected").into());
        }
        Ok(value.into())
    }
    pub fn fit(&self) -> Result<(), JsValue> {
        // Resize callbacks may activate a different terminal on this addon.
        let addon = self.addon.borrow().clone();
        addon
            .fit()
            .map_err(|error| crate::js_error::to_js_error(&error))
    }
    pub fn dispose(&self) {
        self.addon.borrow().dispose();
    }
}
