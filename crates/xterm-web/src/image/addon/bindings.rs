// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use super::{
    handlers::{self, Handlers, ImageHandlersReader},
    option_inspection::{self, ImageOptionsReader},
    options, State,
};
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
    handlers: JsValue,
    options: JsValue,
}
#[wasm_bindgen]
impl ImageAddon {
    #[wasm_bindgen(constructor)]
    pub fn new(options: JsValue) -> Result<Self, JsValue> {
        let options::ConstructorOptions {
            current,
            defaults,
            placeholder,
            storage_limit,
            sixel_size_limit,
        } = options::read(&options)?;
        let options = Rc::new(Cell::new(current));
        let placeholder_value = Rc::new(RefCell::new(placeholder));
        let storage_limit = Rc::new(RefCell::new(storage_limit));
        let option_view = option_inspection::inspection(ImageOptionsReader::new(
            options.clone(),
            placeholder_value.clone(),
            storage_limit.clone(),
            sixel_size_limit.clone(),
        ))?;
        let added = Emitter::new();
        let event = callable(added.event(), empty);
        let handlers = Rc::new(RefCell::new(Handlers::default()));
        let inspection = handlers::inspection(ImageHandlersReader::new(handlers.clone()))?;
        Ok(Self {
            state: Rc::new(State {
                options,
                defaults,
                storage_limit,
                placeholder: Rc::new(Cell::new(current.show_placeholder)),
                placeholder_value,
                sixel_size_limit,
                terminal: RefCell::new(Weak::new()),
                renderer: RefCell::new(None),
                storage: RefCell::new(None),
                inspection: RefCell::new(None),
                cleanups: RefCell::new(Vec::new()),
                handlers,
                added,
            }),
            event,
            handlers: inspection,
            options: option_view,
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
    /// Read-only native handler inspection, not a writable private Map.
    #[wasm_bindgen(getter, js_name = _handlers)]
    pub fn handler_inspection(&self) -> JsValue {
        self.handlers.clone()
    }
    /// Live read-only options. Placeholder and storage-limit values retain their
    /// JavaScript identity; other fields remain typed projections.
    #[wasm_bindgen(getter, js_name = _opts)]
    pub fn option_inspection(&self) -> JsValue {
        self.options.clone()
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
    pub fn set_storage_limit(&self, limit: JsValue) -> Result<(), JsValue> {
        if let Some(storage) = self.state.storage() {
            let pixels = super::storage_limit::pixels(&limit)?;
            storage
                .set_pixel_limit(pixels)
                .map_err(|error| to_js_error(&error))?;
        }
        // Source commits the raw argument only after conversion and eviction.
        // Do not retain a borrow across valueOf, disposal or marker callbacks.
        *self.state.storage_limit.borrow_mut() = limit;
        Ok(())
    }
    #[wasm_bindgen(getter, js_name = storageUsage)]
    pub fn storage_usage(&self) -> Result<f64, JsValue> {
        self.state
            .storage()
            .map_or(Ok(-1.0), |s| s.get_usage().map_err(|e| to_js_error(&e)))
    }
    /// Return the original JavaScript value, not its rendering truthiness.
    #[wasm_bindgen(getter, js_name = showPlaceholder)]
    pub fn show_placeholder(&self) -> JsValue {
        self.state.placeholder_value.borrow().clone()
    }
    /// Store before rendering; host failures do not roll back this value.
    #[wasm_bindgen(setter, js_name = showPlaceholder)]
    pub fn set_show_placeholder(&self, value: JsValue) -> Result<(), JsValue> {
        // Source stores the original value before invoking the renderer. Keep
        // that identity even if rendering throws or a host callback reenters.
        let enabled = value.is_truthy();
        *self.state.placeholder_value.borrow_mut() = value;
        let mut options = self.state.options.get();
        options.show_placeholder = enabled;
        self.state.options.set(options);
        self.state.placeholder.set(enabled);
        if let Some(renderer) = self.state.renderer() {
            renderer.show_placeholder(enabled)?;
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
