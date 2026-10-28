// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Window;
use xterm_addons::search::{SearchAddonError, TimeoutCallback};
use xterm_core::lifecycle::{Disposable, DisposableHandle};

type TimerClosure = Closure<dyn FnMut() -> Result<(), JsValue>>;
struct Timer {
    window: Window,
    id: Cell<Option<i32>>,
    callback: RefCell<Option<TimerClosure>>,
}
impl Disposable for Timer {
    fn dispose(&self) {
        if let Some(id) = self.id.take() {
            self.window.clear_timeout_with_handle(id);
        }
        self.callback.borrow_mut().take();
    }
}
impl Drop for Timer {
    fn drop(&mut self) {
        self.dispose();
    }
}

pub fn timeout(
    window: &Window,
    delay: u64,
    callback: TimeoutCallback,
) -> Result<DisposableHandle, SearchAddonError> {
    let delay = i32::try_from(delay).map_err(|_| SearchAddonError::InvalidCoordinates)?;
    let timer = Rc::new(Timer {
        window: window.clone(),
        id: Cell::new(None),
        callback: RefCell::new(None),
    });
    let weak = Rc::downgrade(&timer);
    let closure = Closure::wrap(Box::new(move || {
        if let Some(timer) = weak.upgrade() {
            timer.id.set(None);
            // wasm-bindgen holds an invocation reference while disposal drops
            // the stored owner, as in the browser task queue implementation.
            callback().map_err(|error| super::error(&error))?;
        }
        Ok(())
    }) as Box<dyn FnMut() -> Result<(), JsValue>>);
    let id = window
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().unchecked_ref(),
            delay,
        )
        .map_err(|value| {
            SearchAddonError::Host(Rc::new(crate::js_error::JsCallbackError(value)))
        })?;
    timer.id.set(Some(id));
    *timer.callback.borrow_mut() = Some(closure);
    Ok(timer)
}
