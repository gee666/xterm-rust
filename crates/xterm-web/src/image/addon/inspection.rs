// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Read-only inspection of the authoritative Rust store, for original assertions
//! and developer diagnostics. This does not expose writable private JS Maps.
use super::{bindings::surface, State};
use crate::image::js;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use xterm_addons::image::storage::{
    CellSize, ImageBufferType, ImageEntries, ImageLayer, ImageSpec, ImageStorage,
};

#[wasm_bindgen(module = "/src/image/addon/inspection.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageStorageInspection)]
    fn storage_inspection(reader: ImageStoreReader) -> Result<JsValue, JsValue>;
}
impl State {
    pub(super) fn inspection(&self) -> JsValue {
        self.inspection
            .borrow()
            .as_ref()
            .cloned()
            .unwrap_or(JsValue::UNDEFINED)
    }
    pub(super) fn inspect_storage(&self, storage: ImageStorage) -> Result<(), JsValue> {
        let view = storage_inspection(ImageStoreReader { storage })?;
        let previous = self.inspection.replace(Some(view));
        drop(previous);
        Ok(())
    }
}

#[wasm_bindgen]
pub struct ImageStoreReader {
    storage: ImageStorage,
}
#[wasm_bindgen]
impl ImageStoreReader {
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> usize {
        self.storage.image_count()
    }
    pub fn has(&self, key: JsValue) -> bool {
        self.spec(&key).is_some()
    }
    pub fn get(&self, key: JsValue) -> JsValue {
        self.spec(&key).map_or(JsValue::UNDEFINED, |spec| {
            ImageSpecInspection { spec }.into()
        })
    }
    pub fn entries(&self) -> ImageStoreIterator {
        ImageStoreIterator {
            entries: self.storage.image_entries(),
        }
    }
}
impl ImageStoreReader {
    fn spec(&self, key: &JsValue) -> Option<Rc<ImageSpec>> {
        let id = key.as_f64()?;
        // Map keys are not numerically coerced. Storage generates exact integer
        // IDs through 2^53; strings, fractions, NaN and other keys cannot exist.
        if !id.is_finite() || id < 0.0 || id > (1_u64 << 53) as f64 || id.fract() != 0.0 {
            return None;
        }
        self.storage.image_spec(id as u64)
    }
}
#[wasm_bindgen]
pub struct ImageStoreIterator {
    entries: ImageEntries,
}
#[wasm_bindgen]
impl ImageStoreIterator {
    pub fn next(&mut self) -> JsValue {
        self.entries
            .next()
            .map_or(JsValue::UNDEFINED, |(id, _)| JsValue::from(id as f64))
    }
}
#[wasm_bindgen]
pub struct ImageSpecInspection {
    spec: Rc<ImageSpec>,
}
#[wasm_bindgen]
impl ImageSpecInspection {
    #[wasm_bindgen(getter)]
    pub fn orig(&self) -> Result<JsValue, JsValue> {
        surface(Some(self.spec.original.clone()))
    }
    #[wasm_bindgen(getter)]
    pub fn actual(&self) -> Result<JsValue, JsValue> {
        surface(self.spec.actual_resource())
    }
    #[wasm_bindgen(getter, js_name = origCellSize)]
    pub fn original_cell_size(&self) -> Result<JsValue, JsValue> {
        cell_size(self.spec.original_cell_size)
    }
    #[wasm_bindgen(getter, js_name = actualCellSize)]
    pub fn actual_cell_size(&self) -> Result<JsValue, JsValue> {
        cell_size(self.spec.actual_cell_size.get())
    }
    #[wasm_bindgen(getter, js_name = tileCount)]
    pub fn tile_count(&self) -> f64 {
        self.spec.tile_count.get()
    }
    #[wasm_bindgen(getter, js_name = bufferType)]
    pub fn buffer_type(&self) -> String {
        match self.spec.buffer_type {
            ImageBufferType::Normal => "normal",
            ImageBufferType::Alternate => "alternate",
        }
        .into()
    }
    #[wasm_bindgen(getter)]
    pub fn layer(&self) -> String {
        match self.spec.layer {
            ImageLayer::Top => "top",
            ImageLayer::Bottom => "bottom",
        }
        .into()
    }
    #[wasm_bindgen(getter, js_name = zIndex)]
    pub fn z_index(&self) -> f64 {
        self.spec.z_index
    }
}
fn cell_size(size: CellSize) -> Result<JsValue, JsValue> {
    let value = js_sys::Object::new();
    js::set(&value, "width", &size.width.into())?;
    js::set(&value, "height", &size.height.into())?;
    Ok(value.into())
}
