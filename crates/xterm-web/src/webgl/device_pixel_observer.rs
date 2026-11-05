// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
//! DevicePixelObserver.ts, including the caller's window realm and callback phases.
use js_sys::{Array, Function, Object, Reflect};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, prelude::wasm_bindgen, JsCast, JsValue};
use web_sys::{HtmlElement, Window};

// A catching operator binding preserves coercion, BigInt and thrown-value identity.
// Observer decisions and lifecycle remain in Rust.
#[wasm_bindgen(
    inline_js = "export function devicePixelPositive(value) { return value > 0; }\nexport function devicePixelField(receiver, key) { return receiver[key]; }"
)]
extern "C" {
    #[wasm_bindgen(catch, js_name = devicePixelPositive)]
    fn positive(value: &JsValue) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = devicePixelField)]
    fn field(receiver: &JsValue, key: &str) -> Result<JsValue, JsValue>;
}

enum Observer {
    Initializing,
    Absent,
    Active(JsValue),
}
struct State {
    observer: RefCell<Observer>,
    disposal_attempted: Cell<bool>,
}
impl State {
    fn value(&self) -> Result<JsValue, JsValue> {
        match &*self.observer.borrow() {
            Observer::Initializing => Err(js_sys::ReferenceError::new(
                "Cannot access 'observer' before initialization",
            )
            .into()),
            Observer::Absent => Ok(JsValue::UNDEFINED),
            Observer::Active(observer) => Ok(observer.clone()),
        }
    }
    fn disconnect(&self, optional: bool) -> Result<(), JsValue> {
        let observer = self.value()?;
        if !optional || !observer.is_undefined() {
            method(&observer, "disconnect")?.call0(&observer)?;
        }
        Ok(())
    }
    fn dispose(&self) -> Result<(), JsValue> {
        // This reference's toDisposable is a bare handler, not an idempotent store.
        self.disposal_attempted.set(true);
        self.disconnect(true)
    }
}

/// The source disposable, with last-owner cleanup for Rust callers. Explicit
/// disposal reports errors on every call and does not suppress an already
/// queued callback. Dropping disconnects and releases the Rust callback state.
/// Retained host callbacks use weak state, so no Rust/JavaScript cycle is leaked.
/// This is the observer component, not a WebglRenderer or WebglAddon binding.
pub struct DevicePixelObserver {
    state: Rc<State>,
}
impl DevicePixelObserver {
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.state.dispose()
    }
}
impl Drop for DevicePixelObserver {
    fn drop(&mut self) {
        // Do not add another host call after explicit disposal, including failure.
        if !self.state.disposal_attempted.get() {
            if let Err(error) = self.state.dispose() {
                crate::write_scheduler::report_unhandled_error(&error);
            }
        }
    }
}

/// Observe physical pixel dimensions using `parent_window.ResizeObserver`, not
/// the ambient constructor. Callback arguments retain their JavaScript values;
/// no rounding or snapshot changes the source's separate width/height reads.
/// Unsupported boxes disconnect; failure of `observe` alone is caught. Failures
/// in construction, disconnect, entry access or callbacks retain their identity.
pub fn observe_device_pixel_dimensions(
    element: &HtmlElement,
    parent_window: &Window,
    callback: &Function,
) -> Result<DevicePixelObserver, JsValue> {
    let state = Rc::new(State {
        observer: RefCell::new(Observer::Initializing),
        disposal_attempted: Cell::new(false),
    });
    let weak = Rc::downgrade(&state);
    let element = element.clone();
    let observed_element = element.clone();
    let callback = callback.clone();
    let notify = Closure::wrap(Box::new(move |entries: JsValue| {
        let Some(state) = weak.upgrade() else {
            return Ok(());
        };
        let target: JsValue = element.clone().into();
        let predicate =
            Closure::wrap(
                Box::new(move |entry: JsValue| Ok(field(&entry, "target")? == target))
                    as Box<dyn Fn(JsValue) -> Result<bool, JsValue>>,
            );
        let entry = method(&entries, "find")?.call1(&entries, &predicate.into_js_value())?;
        if !entry.is_truthy() {
            return Ok(());
        }
        if !Reflect::has(&entry, &"devicePixelContentBoxSize".into())? {
            state.disconnect(true)?;
            *state.observer.borrow_mut() = Observer::Absent;
            return Ok(());
        }
        let width = dimension(&entry, "inlineSize")?;
        let height = dimension(&entry, "blockSize")?;
        if positive(&width)? && positive(&height)? {
            callback.call2(&JsValue::UNDEFINED, &width, &height)?;
        }
        Ok(())
    }) as Box<dyn Fn(JsValue) -> Result<(), JsValue>>);
    let arguments = Array::new();
    // JS owns this closure's lifetime. Its only Rust owner is Weak<State>.
    arguments.push(&notify.into_js_value());
    let constructor = field(parent_window.as_ref(), "ResizeObserver")?
        .dyn_into::<Function>()
        .map_err(|_| js_sys::TypeError::new("ResizeObserver is not a constructor"))?;
    let observer = Reflect::construct(&constructor, &arguments)?;
    *state.observer.borrow_mut() = Observer::Active(observer.clone());
    let observe = || -> Result<(), JsValue> {
        let observe = method(&observer, "observe")?;
        let options = Object::new();
        let boxes = Array::new();
        boxes.push(&"device-pixel-content-box".into());
        Reflect::set(&options, &"box".into(), &boxes)?;
        observe.call2(&observer, observed_element.as_ref(), &options)?;
        Ok(())
    };
    if observe().is_err() {
        // The source catch uses an unconditional disconnect on the live binding.
        // Synchronous unsupported delivery may already have cleared it.
        state.disconnect(false)?;
        *state.observer.borrow_mut() = Observer::Absent;
    }
    Ok(DevicePixelObserver { state })
}
fn method(receiver: &JsValue, name: &str) -> Result<Function, JsValue> {
    field(receiver, name)?
        .dyn_into::<Function>()
        .map_err(|_| js_sys::TypeError::new(&format!("{name} is not a function")).into())
}
fn dimension(entry: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    let boxes = field(entry, "devicePixelContentBoxSize")?;
    let first = field(&boxes, "0")?;
    field(&first, name)
}
