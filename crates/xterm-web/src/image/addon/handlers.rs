// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
//! Ordered native handler ownership, also used by the read-only inspection view.
use super::{Protocol, State};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;

pub(super) struct Entry {
    order: u64,
    protocol: Rc<Protocol>,
}
#[derive(Default)]
pub(super) struct Handlers {
    entries: Vec<Entry>,
    next: u64,
}
impl Handlers {
    pub(super) fn at(&self, index: usize) -> Option<Rc<Protocol>> {
        self.entries.get(index).map(|entry| entry.protocol.clone())
    }
    pub(super) fn clear(&mut self) -> Vec<Entry> {
        // Keep insertion ordinals across clear so retained iterators see appends.
        std::mem::take(&mut self.entries)
    }
}
impl State {
    pub(super) fn add_handler(&self, protocol: Protocol) -> Result<(), JsValue> {
        let retired = {
            let mut handlers = self.handlers.borrow_mut();
            if let Some(entry) = handlers
                .entries
                .iter_mut()
                .find(|p| p.protocol.key() == protocol.key())
            {
                Some(std::mem::replace(&mut entry.protocol, Rc::new(protocol)))
            } else {
                let order = handlers.next;
                handlers.next = order.checked_add(1).ok_or_else(|| {
                    js_sys::RangeError::new("Image handler insertion ordinal exhausted")
                })?;
                handlers.entries.push(Entry {
                    order,
                    protocol: Rc::new(protocol),
                });
                None
            }
        };
        // Releasing a replaced browser protocol may release host resources.
        drop(retired);
        Ok(())
    }
}

#[wasm_bindgen(module = "/src/image/addon/handlers.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = imageHandlerInspection)]
    pub(super) fn inspection(reader: ImageHandlersReader) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen]
pub struct ImageHandlersReader {
    handlers: Rc<RefCell<Handlers>>,
}
impl ImageHandlersReader {
    pub(super) fn new(handlers: Rc<RefCell<Handlers>>) -> Self {
        Self { handlers }
    }
    fn protocol(&self, key: &JsValue) -> Option<Rc<Protocol>> {
        let key = key.as_string()?;
        self.handlers
            .borrow()
            .entries
            .iter()
            .find(|entry| name(&entry.protocol) == key)
            .map(|entry| entry.protocol.clone())
    }
}
#[wasm_bindgen]
impl ImageHandlersReader {
    #[wasm_bindgen(getter)]
    pub fn size(&self) -> usize {
        self.handlers.borrow().entries.len()
    }
    pub fn has(&self, key: JsValue) -> bool {
        self.protocol(&key).is_some()
    }
    pub fn get(&self, key: JsValue) -> JsValue {
        self.protocol(&key).map_or(JsValue::UNDEFINED, |protocol| {
            ImageHandlerInspection { protocol }.into()
        })
    }
    pub fn entries(&self) -> ImageHandlersIterator {
        ImageHandlersIterator {
            handlers: self.handlers.clone(),
            cursor: None,
            done: false,
        }
    }
}
fn name(protocol: &Protocol) -> &'static str {
    match protocol {
        Protocol::Sixel(_) => "sixel",
        Protocol::Iip(_) => "iip",
        Protocol::Kitty(_) => "kitty",
    }
}
#[wasm_bindgen]
pub struct ImageHandlersIterator {
    handlers: Rc<RefCell<Handlers>>,
    cursor: Option<u64>,
    done: bool,
}
#[wasm_bindgen]
impl ImageHandlersIterator {
    pub fn next(&mut self) -> JsValue {
        if self.done {
            return JsValue::UNDEFINED;
        }
        let handlers = self.handlers.borrow();
        if let Some(entry) = handlers
            .entries
            .iter()
            .find(|entry| self.cursor.is_none_or(|cursor| entry.order > cursor))
        {
            self.cursor = Some(entry.order);
            name(&entry.protocol).into()
        } else {
            self.done = true;
            JsValue::UNDEFINED
        }
    }
}
#[wasm_bindgen]
pub struct ImageHandlerInspection {
    protocol: Rc<Protocol>,
}
#[wasm_bindgen]
impl ImageHandlerInspection {
    #[wasm_bindgen(getter)]
    pub fn identity(&self) -> usize {
        Rc::as_ptr(&self.protocol) as usize
    }
    #[wasm_bindgen(getter)]
    pub fn kind(&self) -> String {
        name(&self.protocol).into()
    }
    pub fn reset(&self) -> Result<(), JsValue> {
        self.protocol.reset()
    }
    pub fn kitty(&self) -> Result<JsValue, JsValue> {
        if let Protocol::Kitty(protocol) = self.protocol.as_ref() {
            super::kitty_inspection::inspection(protocol.handler())
        } else {
            Err(js_sys::TypeError::new("Handler is not a Kitty protocol").into())
        }
    }
}
