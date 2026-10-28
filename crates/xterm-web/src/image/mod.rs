// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Browser canvas operations for shared image storage. Terminal lifecycle hooks
//! are separate: callers supply live metrics and DOM ownership through the host.
mod drawing;
mod js;
mod layers;
mod placeholder;
pub mod primitives;
mod resource;
mod storage;

use js::{get, set};
pub use resource::BrowserImageSurface;
use std::rc::Rc;
use wasm_bindgen::JsValue;
use xterm_addons::image::storage::{CellSize, ImageLayer};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct CanvasDimensions {
    pub cell_width: f64,
    pub cell_height: f64,
    pub canvas_width: f64,
    pub canvas_height: f64,
}

/// Each call reads current terminal state. Implementations must not keep a
/// terminal-state borrow across a DOM operation, which can invoke user code.
pub trait ImageCanvasHost {
    fn document(&self) -> Result<JsValue, JsValue>;
    fn screen_element(&self) -> Result<JsValue, JsValue>;
    fn dimensions(&self) -> Result<Option<CanvasDimensions>, JsValue>;
    fn screen_width(&self) -> Result<f64, JsValue>;
}

/// A clone shares layers and placeholder resources. There are no terminal event
/// listeners here; the addon lifecycle owner calls these operations explicitly.
#[derive(Clone)]
pub struct BrowserImageRenderer {
    host: Rc<dyn ImageCanvasHost>,
    layers: js_sys::Map,
    // A JS object lets an outstanding bitmap promise retain only the resource
    // slot, never the terminal or the renderer's Rust owner.
    placeholder: JsValue,
}
fn layer_key(layer: ImageLayer) -> JsValue {
    match layer {
        ImageLayer::Top => "top",
        ImageLayer::Bottom => "bottom",
    }
    .into()
}
impl BrowserImageRenderer {
    pub fn new(host: Rc<dyn ImageCanvasHost>) -> Result<Self, JsValue> {
        let placeholder = js_sys::Object::new().into();
        set(&placeholder, "canvas", &JsValue::UNDEFINED)?;
        set(&placeholder, "bitmap", &JsValue::UNDEFINED)?;
        Ok(Self {
            host,
            layers: js_sys::Map::new(),
            placeholder,
        })
    }

    pub fn cell_size(&self) -> Result<CellSize, JsValue> {
        let dimensions = self.host.dimensions()?.unwrap_or_default();
        Ok(CellSize {
            width: js::number_or(dimensions.cell_width, -1.0),
            height: js::number_or(dimensions.cell_height, -1.0),
        })
    }

    pub fn canvas(&self, layer: ImageLayer) -> Result<Option<JsValue>, JsValue> {
        self.context(layer)
            .map(|context| get(&context, "canvas"))
            .transpose()
    }

    pub fn has_layer(&self, layer: ImageLayer) -> bool {
        self.layers.has(&layer_key(layer))
    }

    fn context(&self, layer: ImageLayer) -> Option<JsValue> {
        let value = self.layers.get(&layer_key(layer));
        (!value.is_undefined()).then_some(value)
    }

    /// Remove both DOM layers and close the current placeholder bitmap.
    /// An already pending bitmap promise follows the source's late assignment
    /// behavior; it is not silently cancelled or claimed to have been closed.
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.remove_layer(ImageLayer::Top)?;
        self.remove_layer(ImageLayer::Bottom)?;
        self.clear_placeholder()
    }
}
