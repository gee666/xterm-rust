//! Read-only packed line projections. Copies do not expose Rust memory to JS GC.
use crate::js_error::to_js_error;
use js_sys::{Object, Reflect, Uint32Array};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::buffer::{buffer_set::BufferRef, BufferLine};

#[wasm_bindgen]
pub struct InternalLines {
    buffer: BufferRef,
}
impl InternalLines {
    pub(super) fn new(buffer: BufferRef) -> Self {
        Self { buffer }
    }
}
#[wasm_bindgen]
impl InternalLines {
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> Result<usize, JsValue> {
        Ok(self
            .buffer
            .try_borrow()
            .map_err(|e| to_js_error(&e))?
            .lines
            .len())
    }
    pub fn get(&self, index: usize) -> Result<JsValue, JsValue> {
        let line = self
            .buffer
            .try_borrow()
            .map_err(|e| to_js_error(&e))?
            .lines
            .get(index);
        Ok(line.map_or(JsValue::UNDEFINED, |native| InternalLine { native }.into()))
    }
}
#[wasm_bindgen]
pub struct InternalLine {
    native: Rc<RefCell<BufferLine>>,
}
#[wasm_bindgen]
impl InternalLine {
    #[wasm_bindgen(getter)]
    pub fn length(&self) -> Result<usize, JsValue> {
        Ok(self.native.try_borrow().map_err(|e| to_js_error(&e))?.len())
    }
    #[wasm_bindgen(getter, js_name = isWrapped)]
    pub fn wrapped(&self) -> Result<bool, JsValue> {
        Ok(self
            .native
            .try_borrow()
            .map_err(|e| to_js_error(&e))?
            .is_wrapped)
    }
    /// An owned packed snapshot, not an unsafe view into reallocating WASM memory.
    #[wasm_bindgen(getter, js_name = _data)]
    pub fn data(&self) -> Result<Uint32Array, JsValue> {
        let packed = self
            .native
            .try_borrow()
            .map_err(|e| to_js_error(&e))?
            .packed_cells()
            .iter()
            .flatten()
            .copied()
            .collect::<Vec<_>>();
        Ok(Uint32Array::from(packed.as_slice()))
    }
    #[wasm_bindgen(getter, js_name = _extendedAttrs)]
    pub fn extended(&self) -> Result<Object, JsValue> {
        let values = {
            let line = self.native.try_borrow().map_err(|e| to_js_error(&e))?;
            let extended = line.extended_attributes();
            extended
                .iter()
                .map(|(index, value)| (*index, value.ext(), value.url_id()))
                .collect::<Vec<_>>()
        };
        let object = Object::new();
        for (index, ext, url) in values {
            let value = Object::new();
            Reflect::set(&value, &"ext".into(), &ext.into())?;
            Reflect::set(&value, &"urlId".into(), &(url as f64).into())?;
            Reflect::set(&object, &index.to_string().into(), &value)?;
        }
        Ok(object)
    }
}
