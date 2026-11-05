// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
//! Diagnostics read native Kitty maps and retain the actual transmitted Blob.
use crate::{image::kitty_encoded::BlobSource, js_error::to_js_error};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use xterm_addons::image::{
    kitty::{
        handler::{KittyGraphicsHandler, PendingTransmission},
        storage::KittyImageData,
    },
    number_map::{NumberMapIterator, NumberMapView},
    storage::StorageError,
};

#[wasm_bindgen(module = "/src/image/addon/handlers.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = kittyHandlerInspection)]
    fn create(
        images: KittyImagesReader,
        pending: KittyPendingReader,
        placements: KittyPlacementsReader,
    ) -> Result<JsValue, JsValue>;
}
pub(super) fn inspection(handler: Rc<KittyGraphicsHandler>) -> Result<JsValue, JsValue> {
    create(
        KittyImagesReader {
            map: handler.images(),
        },
        KittyPendingReader {
            map: handler.pending_transmissions(),
        },
        KittyPlacementsReader {
            map: handler.placements(),
        },
    )
}
fn error(error: StorageError) -> JsValue {
    to_js_error(&error)
}
fn has<T: Clone + 'static>(map: &NumberMapView<T>, key: &JsValue) -> Result<bool, JsValue> {
    key.as_f64().map_or(Ok(false), |key| {
        map.get(key).map(|value| value.is_some()).map_err(error)
    })
}
#[wasm_bindgen]
pub struct KittyImagesReader {
    map: NumberMapView<Rc<KittyImageData>>,
}
#[wasm_bindgen]
impl KittyImagesReader {
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> Result<usize, JsValue> {
        self.map.len().map_err(error)
    }
    pub fn has(&self, key: JsValue) -> Result<bool, JsValue> {
        has(&self.map, &key)
    }
    pub fn get(&self, key: JsValue) -> Result<JsValue, JsValue> {
        let value = key
            .as_f64()
            .map_or(Ok(None), |key| self.map.get(key).map_err(error))?;
        Ok(value.map_or(JsValue::UNDEFINED, |image| {
            KittyImageInspection { image }.into()
        }))
    }
    pub fn entries(&self) -> KittyKeys {
        KittyKeys::new(self.map.iter())
    }
}
#[wasm_bindgen]
pub struct KittyPlacementsReader {
    map: NumberMapView<u64>,
}
#[wasm_bindgen]
impl KittyPlacementsReader {
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> Result<usize, JsValue> {
        self.map.len().map_err(error)
    }
    pub fn has(&self, key: JsValue) -> Result<bool, JsValue> {
        has(&self.map, &key)
    }
    pub fn get(&self, key: JsValue) -> Result<JsValue, JsValue> {
        let value = key
            .as_f64()
            .map_or(Ok(None), |key| self.map.get(key).map_err(error))?;
        Ok(value.map_or(JsValue::UNDEFINED, |id| (id as f64).into()))
    }
    pub fn entries(&self) -> KittyKeys {
        KittyKeys::new(self.map.iter())
    }
}
/// Membership-only diagnostics. Pending command/decoder objects are deliberately
/// not presented as detached snapshots of the source's mutable records.
#[wasm_bindgen]
pub struct KittyPendingReader {
    map: NumberMapView<Rc<PendingTransmission>>,
}
#[wasm_bindgen]
impl KittyPendingReader {
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> Result<usize, JsValue> {
        self.map.len().map_err(error)
    }
    pub fn has(&self, key: JsValue) -> Result<bool, JsValue> {
        has(&self.map, &key)
    }
    pub fn entries(&self) -> KittyKeys {
        KittyKeys::new(self.map.iter())
    }
}
#[wasm_bindgen]
pub struct KittyKeys {
    keys: Box<dyn Iterator<Item = Result<f64, StorageError>>>,
}
impl KittyKeys {
    fn new<T: 'static>(iter: NumberMapIterator<T>) -> Self {
        Self {
            keys: Box::new(iter.map(|entry| entry.map(|(key, _)| key))),
        }
    }
}
#[wasm_bindgen]
impl KittyKeys {
    pub fn next(&mut self) -> Result<JsValue, JsValue> {
        self.keys
            .next()
            .transpose()
            .map(|key| key.map_or(JsValue::UNDEFINED, JsValue::from))
            .map_err(error)
    }
}
#[wasm_bindgen]
pub struct KittyImageInspection {
    image: Rc<KittyImageData>,
}
#[wasm_bindgen]
impl KittyImageInspection {
    #[wasm_bindgen(getter)]
    pub fn identity(&self) -> usize {
        Rc::as_ptr(&self.image) as usize
    }
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> f64 {
        self.image.id.get()
    }
    #[wasm_bindgen(getter)]
    pub fn width(&self) -> f64 {
        self.image.bytes.borrow().width
    }
    #[wasm_bindgen(getter)]
    pub fn height(&self) -> f64 {
        self.image.bytes.borrow().height
    }
    #[wasm_bindgen(getter)]
    pub fn format(&self) -> f64 {
        self.image.bytes.borrow().format
    }
    #[wasm_bindgen(getter)]
    pub fn compression(&self) -> js_sys::JsString {
        js_sys::JsString::from_char_code(&self.image.bytes.borrow().compression)
    }
    #[wasm_bindgen(getter)]
    pub fn data(&self) -> Result<JsValue, JsValue> {
        self.image
            .bytes
            .borrow()
            .data
            .source::<BlobSource>()
            .map(BlobSource::value)
            .ok_or_else(|| {
                js_sys::TypeError::new("Kitty image has a non-browser byte source").into()
            })
    }
}
