// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::dom_helpers::get_window;
use js_sys::{Function, Reflect};
use std::{cell::{Cell, RefCell}, rc::{Rc, Weak}};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Node, Window};

struct State {
    fallback: Window,
    default_target: Option<Window>,
    disposed: Cell<bool>,
    pending: RefCell<Option<(Window, JsValue)>>,
}
impl State {
    fn cancel(&self) -> Result<(), JsValue> {
        let pending = self.pending.borrow().clone();
        if let Some((window, handle)) = pending {
            let clear: Function = Reflect::get(window.as_ref(), &"clearInterval".into())?.dyn_into()?;
            clear.call1(window.as_ref(), &handle)?;
            self.pending.borrow_mut().take();
        }
        Ok(())
    }
}
impl Drop for State {
    fn drop(&mut self) {
        if let Err(error) = self.cancel() { web_sys::console::error_1(&error); }
    }
}

/// Intervals use the captured node's window unless explicitly overridden.
/// Each JS callback owns its original runner, independently of later cancellation
/// or replacement. It does not capture this timer and create a reference cycle.
#[derive(Clone)]
pub struct WindowIntervalTimer { state: Rc<State> }
#[derive(Clone)]
pub struct WeakWindowIntervalTimer { state: Weak<State> }
impl WeakWindowIntervalTimer {
    pub fn upgrade(&self) -> Option<WindowIntervalTimer> {
        self.state.upgrade().map(|state| WindowIntervalTimer { state })
    }
}
impl WindowIntervalTimer {
    pub fn new(node: Option<&Node>, fallback: Window) -> Self {
        let default_target = node.map(|node| get_window(Some(node), None, &fallback));
        Self { state: Rc::new(State { fallback, default_target, disposed: Cell::new(false), pending: RefCell::new(None) }) }
    }
    pub fn downgrade(&self) -> WeakWindowIntervalTimer {
        WeakWindowIntervalTimer { state: Rc::downgrade(&self.state) }
    }
    pub fn cancel_and_set(&self, runner: impl Fn() -> Result<(), JsValue> + 'static,
        interval_ms: f64, target: Option<&Window>) -> Result<(), JsValue> {
        if self.state.disposed.get() {
            return Err(js_sys::Error::new("Calling cancelAndSet on a disposed IntervalTimer").into());
        }
        self.cancel()?;
        let window = target.or(self.state.default_target.as_ref()).unwrap_or(&self.state.fallback);
        let set: Function = Reflect::get(window.as_ref(), &"setInterval".into())?.dyn_into()?;
        // JS ownership lets an instrumented host retain and invoke an old
        // callback after clearInterval, exactly like the source's plain closure.
        // Fn, rather than FnMut, also permits synchronous callback reentrancy.
        let callback = Closure::wrap(Box::new(runner) as Box<dyn Fn() -> Result<(), JsValue>>).into_js_value();
        let handle = set.call2(window.as_ref(), &callback, &JsValue::from_f64(interval_ms))?;
        let old = self.state.pending.replace(Some((window.clone(), handle)));
        drop(old);
        Ok(())
    }
    pub fn cancel(&self) -> Result<(), JsValue> { self.state.cancel() }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.cancel()?;
        self.state.disposed.set(true);
        Ok(())
    }
}
