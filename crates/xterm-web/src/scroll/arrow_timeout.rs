// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Source TimeoutTimer semantics for arrow activation. Unlike a numeric browser
//! scheduler, this keeps raw host tokens and a separate runner per request.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};

#[wasm_bindgen(inline_js = "
export function arrowSetTimeout(runner, delay) { return setTimeout(runner, delay); }
export function arrowClearTimeout(token) { clearTimeout(token); }
")]
extern "C" {
    #[wasm_bindgen(catch,js_name=arrowSetTimeout)]
    fn set_timeout(runner: &js_sys::Function, delay: f64) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch,js_name=arrowClearTimeout)]
    fn clear_timeout(token: &JsValue) -> Result<(), JsValue>;
}
struct State {
    token: RefCell<JsValue>,
    disposed: Cell<bool>,
}
#[derive(Clone)]
pub(super) struct ArrowTimeout {
    state: Rc<State>,
}
impl ArrowTimeout {
    pub(super) fn new() -> Self {
        Self {
            state: Rc::new(State {
                token: RefCell::new((-1).into()),
                disposed: Cell::new(false),
            }),
        }
    }
    pub(super) fn cancel(&self) -> Result<(), JsValue> {
        let token = self.state.token.borrow().clone();
        if token != JsValue::from_f64(-1.0) {
            clear_timeout(&token)?;
            self.state.token.replace((-1).into());
        }
        Ok(())
    }
    pub(super) fn cancel_and_set(
        &self,
        runner: Rc<dyn Fn() -> Result<(), JsValue>>,
        delay: f64,
    ) -> Result<(), JsValue> {
        if self.state.disposed.get() {
            return Err(
                js_sys::Error::new("Calling cancelAndSet on a disposed TimeoutTimer").into(),
            );
        }
        self.cancel()?;
        let state = self.state.clone();
        let callback = Closure::wrap(Box::new(move || {
            state.token.replace((-1).into());
            runner()
        }) as Box<dyn Fn() -> Result<(), JsValue>>)
        .into_js_value();
        let token = set_timeout(callback.unchecked_ref(), delay)?;
        self.state.token.replace(token);
        Ok(())
    }
    pub(super) fn dispose(&self) -> Result<(), JsValue> {
        self.cancel()?;
        self.state.disposed.set(true);
        Ok(())
    }
}
