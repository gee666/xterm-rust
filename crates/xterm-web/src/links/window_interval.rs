// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::dom_helpers::get_window;
use js_sys::Function;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Node, Window};

type Runner = Rc<dyn Fn(&WindowIntervalTimer) -> Result<(), JsValue>>;
struct State {
    fallback: Window,
    default_target: Option<Window>,
    disposed: Cell<bool>,
    pending: RefCell<Option<(Window, JsValue)>>,
    runner: RefCell<Option<Runner>>,
    callback: RefCell<Option<Closure<dyn FnMut()>>>,
}
impl State {
    fn cancel(&self) -> Result<(), JsValue> {
        let pending = self.pending.borrow().clone();
        if let Some((window, handle)) = pending {
            let clear: Function =
                js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("clearInterval"))?
                    .dyn_into()?;
            clear.call1(window.as_ref(), &handle)?;
            self.pending.borrow_mut().take();
            let runner = self.runner.borrow_mut().take();
            drop(runner);
        }
        Ok(())
    }
}
impl Drop for State {
    fn drop(&mut self) {
        if let Err(error) = self.cancel() {
            web_sys::console::error_1(&error);
        }
        self.callback.get_mut().take();
    }
}

/// An interval in the owning document's window, unless explicitly overridden.
/// The permanent closure captures weak state, allowing cancellation from within
/// the runner without releasing the currently executing callback allocation.
#[derive(Clone)]
pub struct WindowIntervalTimer {
    state: Rc<State>,
}
impl WindowIntervalTimer {
    pub fn new(node: Option<&Node>, fallback: Window) -> Self {
        let default_target = node.map(|node| get_window(Some(node), None, &fallback));
        let state = Rc::new(State {
            fallback,
            default_target,
            disposed: Cell::new(false),
            pending: RefCell::new(None),
            runner: RefCell::new(None),
            callback: RefCell::new(None),
        });
        let weak = Rc::downgrade(&state);
        let callback = Closure::wrap(Box::new(move || {
            let Some(state) = weak.upgrade() else { return };
            let runner = state.runner.borrow().clone();
            if let Some(runner) = runner {
                if let Err(error) = runner(&WindowIntervalTimer {
                    state: Rc::clone(&state),
                }) {
                    web_sys::console::error_1(&error);
                }
            }
        }) as Box<dyn FnMut()>);
        *state.callback.borrow_mut() = Some(callback);
        Self { state }
    }
    pub fn cancel_and_set(
        &self,
        runner: impl Fn(&Self) -> Result<(), JsValue> + 'static,
        interval_ms: f64,
        target: Option<&Window>,
    ) -> Result<(), JsValue> {
        if self.state.disposed.get() {
            return Err(JsValue::from_str(
                "Calling cancelAndSet on a disposed IntervalTimer",
            ));
        }
        self.cancel()?;
        let window = target
            .or(self.state.default_target.as_ref())
            .unwrap_or(&self.state.fallback);
        let set: Function =
            js_sys::Reflect::get(window.as_ref(), &JsValue::from_str("setInterval"))?.dyn_into()?;
        let callback = self.state.callback.borrow();
        let callback = callback
            .as_ref()
            .ok_or_else(|| JsValue::from_str("interval callback unavailable"))?;
        let handle = set.call2(
            window.as_ref(),
            callback.as_ref().unchecked_ref(),
            &JsValue::from_f64(interval_ms),
        )?;
        *self.state.runner.borrow_mut() = Some(Rc::new(runner));
        *self.state.pending.borrow_mut() = Some((window.clone(), handle));
        Ok(())
    }
    pub fn cancel(&self) -> Result<(), JsValue> {
        self.state.cancel()
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.cancel()?;
        self.state.disposed.set(true);
        Ok(())
    }
}
