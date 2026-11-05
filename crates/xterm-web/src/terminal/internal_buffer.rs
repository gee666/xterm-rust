//! Internal active-buffer views retain native buffer identity, not field copies.
use crate::js_error::to_js_error;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use xterm_core::buffer::{buffer_set::BufferRef, Buffer, BufferService};

struct CachedBuffer {
    native: BufferRef,
    object: JsValue,
}
#[wasm_bindgen]
pub struct InternalBufferServiceView {
    native: Rc<BufferService>,
    normal: RefCell<Option<CachedBuffer>>,
    alternate: RefCell<Option<CachedBuffer>>,
}
impl InternalBufferServiceView {
    pub(super) fn new(native: Rc<BufferService>) -> Self {
        Self {
            native,
            normal: RefCell::new(None),
            alternate: RefCell::new(None),
        }
    }
}
#[wasm_bindgen]
impl InternalBufferServiceView {
    #[wasm_bindgen(getter)]
    pub fn cols(&self) -> usize {
        self.native.cols()
    }
    #[wasm_bindgen(getter)]
    pub fn rows(&self) -> usize {
        self.native.rows()
    }
    #[wasm_bindgen(getter)]
    pub fn buffer(&self) -> JsValue {
        let native = self.native.buffer();
        let slot = if Rc::ptr_eq(&native, &self.native.buffers.normal()) {
            &self.normal
        } else {
            &self.alternate
        };
        if let Some(cached) = slot.borrow().as_ref() {
            if Rc::ptr_eq(&native, &cached.native) {
                return cached.object.clone();
            }
        }
        let object: JsValue = InternalBufferView {
            native: Rc::clone(&native),
        }
        .into();
        let previous = slot.replace(Some(CachedBuffer {
            native,
            object: object.clone(),
        }));
        drop(previous);
        object
    }
}
#[wasm_bindgen]
pub struct InternalBufferView {
    native: BufferRef,
}
impl InternalBufferView {
    fn read(&self, read: impl FnOnce(&Buffer) -> usize) -> Result<usize, JsValue> {
        let buffer = self
            .native
            .try_borrow()
            .map_err(|error| to_js_error(&error))?;
        Ok(read(&buffer))
    }
}
#[wasm_bindgen]
impl InternalBufferView {
    #[wasm_bindgen(getter, js_name = scrollTop)]
    pub fn scroll_top(&self) -> Result<usize, JsValue> {
        self.read(|buffer| buffer.scroll_top)
    }
    #[wasm_bindgen(getter, js_name = scrollBottom)]
    pub fn scroll_bottom(&self) -> Result<usize, JsValue> {
        self.read(|buffer| buffer.scroll_bottom)
    }
    #[wasm_bindgen(getter)]
    pub fn x(&self) -> Result<usize, JsValue> {
        self.read(|buffer| buffer.x)
    }
    #[wasm_bindgen(getter)]
    pub fn y(&self) -> Result<usize, JsValue> {
        self.read(|buffer| buffer.y)
    }
    #[wasm_bindgen(getter)]
    pub fn ybase(&self) -> Result<usize, JsValue> {
        self.read(|buffer| buffer.ybase)
    }
    #[wasm_bindgen(getter)]
    pub fn ydisp(&self) -> Result<usize, JsValue> {
        self.read(|buffer| buffer.ydisp)
    }
}
