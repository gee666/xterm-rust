// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{options, State};
use crate::{
    image::BrowserImageSurface,
    js_error::to_js_error,
    terminal::{
        events::{callable, empty},
        Terminal,
    },
};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use wasm_bindgen::prelude::*;
use xterm_addons::image::storage::ImageResource;
use xterm_core::event::Emitter;

/// Image protocols, control sequences and image storage for the Rust terminal.
#[wasm_bindgen]
pub struct ImageAddon {
    state: Rc<State>,
    event: js_sys::Function,
}
#[wasm_bindgen]
impl ImageAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(options: JsValue) -> Result<Self, JsValue> {
        let current = options::read(&options)?;
        let defaults = options::read(&options)?;
        let added = Emitter::new();
        let event = callable(added.event(), empty);
        Ok(Self {
            state: Rc::new(State {
                options: Rc::new(Cell::new(current)),
                defaults,
                placeholder: Rc::new(Cell::new(current.show_placeholder)),
                terminal: RefCell::new(Weak::new()),
                renderer: RefCell::new(None),
                storage: RefCell::new(None),
                inspection: RefCell::new(None),
                cleanups: RefCell::new(Vec::new()),
                handlers: RefCell::new(Vec::new()),
                added,
            }),
            event,
        })
    }
    pub fn activate(&self, terminal: &Terminal) -> Result<(), JsValue> {
        self.state.activate(terminal)
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.state.dispose()
    }
    pub fn reset(&self) -> Result<bool, JsValue> {
        self.state.reset()
    }
    /// Live read-only storage inspection. Writable source-private fields are not
    /// part of this binding; image decoding and storage remain authoritative Rust.
    #[wasm_bindgen(getter, js_name = _storage)]
    pub fn storage_inspection(&self) -> JsValue {
        self.state.inspection()
    }
    #[wasm_bindgen(getter, js_name = onImageAdded)]
    pub fn on_image_added(&self) -> js_sys::Function {
        self.event.clone()
    }
    #[wasm_bindgen(getter, js_name = storageLimit)]
    pub fn storage_limit(&self) -> f64 {
        self.state.storage().map_or(-1.0, |s| {
            let value = s.get_limit();
            if value == 0.0 || value.is_nan() {
                -1.0
            } else {
                value
            }
        })
    }
    #[wasm_bindgen(setter, js_name = storageLimit)]
    pub fn set_storage_limit(&self, limit: f64) -> Result<(), JsValue> {
        if let Some(storage) = self.state.storage() {
            storage.set_limit(limit).map_err(|error| match error {
                xterm_addons::image::storage::StorageError::InvalidLimit => {
                    js_sys::RangeError::new(&error.to_string()).into()
                }
                _ => to_js_error(&error),
            })?;
        }
        let mut options = self.state.options.get();
        options.storage_limit = limit;
        self.state.options.set(options);
        Ok(())
    }
    #[wasm_bindgen(getter, js_name = storageUsage)]
    pub fn storage_usage(&self) -> Result<f64, JsValue> {
        self.state
            .storage()
            .map_or(Ok(-1.0), |s| s.get_usage().map_err(|e| to_js_error(&e)))
    }
    #[wasm_bindgen(getter, js_name = showPlaceholder)]
    pub fn show_placeholder(&self) -> bool {
        self.state.options.get().show_placeholder
    }
    #[wasm_bindgen(setter, js_name = showPlaceholder)]
    pub fn set_show_placeholder(&self, value: bool) -> Result<(), JsValue> {
        let mut options = self.state.options.get();
        options.show_placeholder = value;
        self.state.options.set(options);
        self.state.placeholder.set(value);
        if let Some(renderer) = self.state.renderer() {
            renderer.show_placeholder(value)?;
        }
        Ok(())
    }
    #[wasm_bindgen(js_name = getImageAtBufferCell)]
    pub fn get_image_at_buffer_cell(&self, x: f64, y: f64) -> Result<JsValue, JsValue> {
        self.state.storage().map_or(Ok(JsValue::UNDEFINED), |s| {
            surface(
                s.get_image_at_buffer_cell_number(x, y)
                    .map_err(|e| to_js_error(&e))?,
            )
        })
    }
    #[wasm_bindgen(js_name = extractTileAtBufferCell)]
    pub fn extract_tile_at_buffer_cell(&self, x: f64, y: f64) -> Result<JsValue, JsValue> {
        self.state.storage().map_or(Ok(JsValue::UNDEFINED), |s| {
            surface(
                s.extract_tile_at_buffer_cell_number(x, y)
                    .map_err(|e| to_js_error(&e))?,
            )
        })
    }
}
pub(super) fn surface(resource: Option<ImageResource>) -> Result<JsValue, JsValue> {
    resource.map_or(Ok(JsValue::UNDEFINED), |image| {
        image
            .surface::<BrowserImageSurface>()
            .map(|surface| surface.value())
            .ok_or_else(|| {
                js_sys::TypeError::new("Image storage contains a non-browser surface").into()
            })
    })
}
