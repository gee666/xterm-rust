//! Fallible host reads keep original JavaScript exceptions across the WASM edge.
use js_sys::{Array, Function, Reflect};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen(module = "/src/core_browser_microtask.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = queueFocusCache)]
    pub(super) fn queue_cache_reset(callback: &JsValue) -> Result<(), JsValue>;
}

pub(super) fn invoke0(target: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    let method = Reflect::get(target, &JsValue::from_str(name))?;
    // Reflect.apply validates callability and retains host-thrown values. A Rust
    // brand check would instead return the non-callable value as an exception.
    Reflect::apply(method.unchecked_ref::<Function>(), target, &Array::new())
}

pub(super) fn invoke1(
    target: &JsValue,
    name: &str,
    argument: impl FnOnce() -> Result<JsValue, JsValue>,
) -> Result<JsValue, JsValue> {
    // JS resolves the method before evaluating its argument. Reentrant getters
    // can change the monitor's window/callback during either step.
    let method = Reflect::get(target, &JsValue::from_str(name))?;
    let args = Array::new();
    args.push(&argument()?);
    Reflect::apply(method.unchecked_ref::<Function>(), target, &args)
}

pub(super) fn number(target: &JsValue, name: &str) -> Result<f64, JsValue> {
    Reflect::get(target, &JsValue::from_str(name))?
        .as_f64()
        .ok_or_else(|| js_sys::TypeError::new(&format!("{name} must be a number")).into())
}

/// Default-argument DomListener with shared identity for MutableDisposable
/// reentry. The general DOM token requires &mut and a capture bool; this source
/// path must retain the old token across host calls and pass undefined options.
pub(super) struct DefaultDomListener {
    target: JsValue,
    kind: String,
    callback: Function,
    active: Cell<bool>,
    removal_failed: Cell<bool>,
}
impl DefaultDomListener {
    pub(super) fn new(target: &JsValue, kind: &str, callback: Function) -> Rc<Self> {
        Rc::new(Self {
            target: target.clone(),
            kind: kind.into(),
            callback,
            active: Cell::new(false),
            removal_failed: Cell::new(false),
        })
    }
    fn invoke(&self, method: &str, registering: bool) -> Result<(), JsValue> {
        let callable = Reflect::get(&self.target, &JsValue::from_str(method))?;
        // A throwing method getter cannot have registered this callback. Once
        // invocation starts, a throwing host may already have accepted it.
        if registering {
            self.active.set(true);
        }
        let args = Array::new();
        args.push(&JsValue::from_str(&self.kind));
        args.push(&if self.active.get() {
            self.callback.clone().into()
        } else {
            JsValue::NULL
        });
        args.push(&JsValue::UNDEFINED);
        Reflect::apply(callable.unchecked_ref::<Function>(), &self.target, &args)?;
        Ok(())
    }
    pub(super) fn add(&self) -> Result<(), JsValue> {
        self.invoke("addEventListener", true)
    }
    pub(super) fn remove(&self) -> Result<(), JsValue> {
        if !self.active.get() {
            return Ok(());
        }
        let result = self.invoke("removeEventListener", false);
        self.removal_failed.set(result.is_err());
        result?;
        self.active.set(false);
        Ok(())
    }
    pub(super) fn cleanup(&self) {
        if self.active.get() && !self.removal_failed.get() {
            if let Err(error) = self.remove() {
                report_drop_error(&error);
            }
        }
    }
}

pub(super) fn report_drop_error(error: &JsValue) {
    // Drop has no JavaScript caller to return to. Explicit disposal is fallible;
    // last-owner cleanup reports failures rather than panicking across FFI.
    crate::write_scheduler::report_unhandled_error(error);
}
