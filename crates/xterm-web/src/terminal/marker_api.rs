// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
//! Public marker handles share the native buffer's identity and lifecycle.
use super::{events, Terminal};
use crate::{js_error::to_js_error, utf16};
use js_sys::{Array, Function, JsString};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::buffer::{marker::Marker, marker_line::MarkerLine};

// Validation deliberately operates on the original argument twice, just like
// the source public wrapper. Addition happens later, after capturing the buffer.
#[wasm_bindgen(inline_js = "
export function validateMarkerOffset(value) {
  if (value === undefined) value = 0;
  if (value === Infinity || isNaN(value) || value % 1 !== 0) {
    throw new Error('This API only accepts integers');
  }
  return value;
}
export function markerPosition(base, offset) { return base + offset; }
export function markerWeakRef(value) { return new WeakRef(value); }
export function markerDeref(value) { return value.deref(); }
export function markerList(read) {
  const target = [];
  const sync = () => {
    const values = read();
    target.length = values.length;
    for (let i = 0; i < values.length; i++) target[i] = values[i];
  };
  const readonly = () => { throw new TypeError('Marker list is a read-only view'); };
  return new Proxy(target, {
    get(target, key, receiver) { sync(); return Reflect.get(target, key, receiver); },
    has(target, key) { sync(); return Reflect.has(target, key); },
    ownKeys(target) { sync(); return Reflect.ownKeys(target); },
    getOwnPropertyDescriptor(target, key) { sync(); return Reflect.getOwnPropertyDescriptor(target, key); },
    set: readonly, deleteProperty: readonly, defineProperty: readonly
  });
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = validateMarkerOffset)]
    fn validate_offset(value: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = markerPosition)]
    fn position(base: f64, offset: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = markerWeakRef)]
    fn weak_ref(value: &JsValue) -> JsValue;
    #[wasm_bindgen(js_name = markerDeref)]
    fn deref(value: &JsValue) -> JsValue;
    #[wasm_bindgen(js_name = markerList)]
    fn marker_list(read: JsValue) -> Array;
}

/// Weak JS references preserve observable identity without retaining disposed
/// marker wrappers for the lifetime of an otherwise long-lived Terminal.
#[derive(Default)]
pub(crate) struct MarkerHandles(RefCell<Vec<(Marker, JsValue)>>);
impl MarkerHandles {
    fn get(&self, marker: Marker) -> JsValue {
        let mut handles = self.0.borrow_mut();
        handles.retain(|(_, weak)| !deref(weak).is_undefined());
        if let Some((_, weak)) = handles.iter().find(|(native, _)| native.ptr_eq(&marker)) {
            return deref(weak);
        }
        let value: JsValue = BrowserMarker::new(marker.clone()).into();
        handles.push((marker, weak_ref(&value)));
        value
    }
}

type NativeMarkers = Rc<RefCell<Vec<Marker>>>;
#[derive(Default)]
pub(crate) struct MarkerLists(RefCell<Vec<(NativeMarkers, JsValue)>>);
impl MarkerLists {
    fn get(&self, markers: NativeMarkers, handles: Rc<MarkerHandles>) -> Array {
        let mut lists = self.0.borrow_mut();
        lists.retain(|(_, weak)| !deref(weak).is_undefined());
        if let Some((_, weak)) = lists
            .iter()
            .find(|(native, _)| Rc::ptr_eq(native, &markers))
        {
            return deref(weak).unchecked_into();
        }
        let native = Rc::clone(&markers);
        // The proxy owns this closure, not vice versa. Only a WeakRef is stored
        // in the cache, so retaining a list need not retain the Terminal.
        let read = Closure::<dyn Fn() -> Result<Array, JsValue>>::new(move || {
            let snapshot = native
                .try_borrow()
                .map_err(|error| to_js_error(&error))?
                .clone();
            let result = Array::new();
            for marker in snapshot {
                result.push(&handles.get(marker));
            }
            Ok(result)
        })
        .into_js_value();
        let list = marker_list(read);
        lists.push((markers, weak_ref(list.as_ref())));
        list
    }
}

#[wasm_bindgen(js_name = Marker)]
pub struct BrowserMarker {
    marker: Marker,
    on_dispose: Function,
}
impl BrowserMarker {
    fn new(marker: Marker) -> Self {
        let on_dispose = events::callable(marker.on_dispose(), events::empty);
        Self { marker, on_dispose }
    }
}
#[wasm_bindgen(js_class = Marker)]
impl BrowserMarker {
    #[wasm_bindgen(getter)]
    pub fn id(&self) -> f64 {
        self.marker.id() as f64
    }
    #[wasm_bindgen(getter)]
    pub fn line(&self) -> JsValue {
        let line = self.marker.line_value();
        if let Some(number) = line.as_number() {
            number.into()
        } else {
            utf16::to_js_string(&line.to_utf16()).into()
        }
    }
    #[wasm_bindgen(getter, js_name = isDisposed)]
    pub fn is_disposed(&self) -> bool {
        self.marker.is_disposed()
    }
    #[wasm_bindgen(getter, js_name = onDispose)]
    pub fn on_dispose(&self) -> Function {
        self.on_dispose.clone()
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.marker.dispose().map_err(|error| to_js_error(&error))
    }
}

#[wasm_bindgen]
impl Terminal {
    #[wasm_bindgen(js_name = registerMarker)]
    pub fn register_marker(&self, offset: JsValue) -> Result<JsValue, JsValue> {
        let offset = validate_offset(offset)?;
        // Coercion can synchronously switch/reset buffers. Capture the receiver
        // before evaluating the addition, and never borrow it during user code.
        let buffer = self
            .shared
            .native
            .with_state(|state| state.active_buffer())
            .map_err(|error| to_js_error(&error))?;
        let base = {
            let buffer = buffer.try_borrow().map_err(|error| to_js_error(&error))?;
            (buffer.ybase + buffer.y) as f64
        };
        let value = position(base, offset)?;
        let line = if let Some(number) = value.as_f64() {
            MarkerLine::number(number)
        } else {
            // ECMAScript addition of a Number yields either Number or String,
            // or throws before reaching this conversion, e.g. BigInt/Symbol.
            MarkerLine::string(utf16::from_js_string(&value.unchecked_into::<JsString>()))
        };
        let marker = buffer
            .try_borrow_mut()
            .map_err(|error| to_js_error(&error))?
            .add_marker_value(line);
        Ok(self.shared.marker_handles.get(marker))
    }

    /// A live read-only Array view, preserving list and marker handle identity.
    /// Source's writable Array operations remain unsupported.
    #[wasm_bindgen(getter)]
    pub fn markers(&self) -> Result<Array, JsValue> {
        let buffer = self
            .shared
            .native
            .with_state(|state| state.active_buffer())
            .map_err(|error| to_js_error(&error))?;
        let markers = Rc::clone(
            &buffer
                .try_borrow()
                .map_err(|error| to_js_error(&error))?
                .markers,
        );
        Ok(self
            .shared
            .marker_lists
            .get(markers, Rc::clone(&self.shared.marker_handles)))
    }
}
