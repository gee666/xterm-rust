// Copyright (c) 2023 The xterm.js authors. MIT license.
//! A browser-owned memory mirror preserves source QOI view aliasing. The codec
//! remains Rust; platform Memory supplies buffer identity and grow detachment.
use super::js::{call, function, get, set};
use js_sys::{Array, Reflect, Uint8Array};
use std::cell::RefCell;
use wasm_bindgen::JsValue;

const PAGE_BYTES: usize = 65_536;

#[derive(Default)]
pub(super) struct QoiMemory(RefCell<Option<JsValue>>);
impl QoiMemory {
    pub fn snapshot(&self) -> Result<Option<Vec<u8>>, JsValue> {
        let memory = self.0.borrow().clone();
        memory
            .map(|memory| Ok(Uint8Array::new(&get(&memory, "buffer")?).to_vec()))
            .transpose()
    }
    pub fn clear(&self) {
        self.0.borrow_mut().take();
    }
    pub fn update(&self, bytes: &[u8]) -> Result<JsValue, JsValue> {
        let previous = self.0.borrow().clone();
        let memory = if let Some(memory) = previous {
            let size = Uint8Array::new(&get(&memory, "buffer")?).length() as usize;
            if bytes.len() > size {
                // Memory.grow detaches existing typed views, unlike replacing an
                // ArrayBuffer. Nested ImageData construction can observe this.
                call(
                    &memory,
                    "grow",
                    &[((bytes.len() - size) / PAGE_BYTES).into()],
                )?;
            }
            memory
        } else {
            let descriptor = js_sys::Object::new();
            set(&descriptor, "initial", &(bytes.len() / PAGE_BYTES).into())?;
            let constructor = function(
                get(&get(&js_sys::global(), "WebAssembly")?, "Memory")?,
                "WebAssembly.Memory",
            )?;
            let memory = Reflect::construct(&constructor, &Array::of1(&descriptor))?;
            *self.0.borrow_mut() = Some(memory.clone());
            memory
        };
        let buffer = get(&memory, "buffer")?;
        Uint8Array::new(&buffer).copy_from(bytes);
        Ok(buffer)
    }
}
