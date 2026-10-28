// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use super::{js_max, js_min};
use crate::links::debounce::RenderRange;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Window;
use xterm_core::core_service::CoreService;
const SYNCHRONIZED_OUTPUT_TIMEOUT_MS: i32 = 1000;
type Callback = Closure<dyn Fn() -> Result<(), JsValue>>;
struct State {
    window: Rc<dyn Fn() -> Window>,
    core: Rc<CoreService>,
    range: Cell<RenderRange>,
    buffering: Cell<bool>,
    timeout: Cell<Option<i32>>,
    callback: RefCell<Option<Callback>>,
    on_timeout: Rc<dyn Fn() -> Result<(), JsValue>>,
}
impl State {
    fn cancel_timer(&self) {
        if let Some(handle) = self.timeout.take() {
            (self.window)().clear_timeout_with_handle(handle);
        }
        self.callback.borrow_mut().take();
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.cancel_timer();
    }
}
#[derive(Clone)]
pub(super) struct SynchronizedOutput(Rc<State>);
impl SynchronizedOutput {
    pub fn new(
        window: Rc<dyn Fn() -> Window>,
        core: Rc<CoreService>,
        on_timeout: impl Fn() -> Result<(), JsValue> + 'static,
    ) -> Self {
        Self(Rc::new(State {
            window,
            core,
            range: Cell::new(RenderRange {
                start: 0.0,
                end: 0.0,
            }),
            buffering: Cell::new(false),
            timeout: Cell::new(None),
            callback: RefCell::new(None),
            on_timeout: Rc::new(on_timeout),
        }))
    }
    pub fn buffer_rows(&self, start: f64, end: f64) -> Result<(), JsValue> {
        let range = if self.0.buffering.replace(true) {
            let old = self.0.range.get();
            RenderRange {
                start: js_min(old.start, start),
                end: js_max(old.end, end),
            }
        } else {
            RenderRange { start, end }
        };
        self.0.range.set(range);
        if self.0.timeout.get().is_none() {
            let weak = Rc::downgrade(&self.0);
            let callback: Callback = Closure::wrap(Box::new(move || {
                if let Some(state) = weak.upgrade() {
                    state.timeout.set(None);
                    state
                        .core
                        .dec_private_modes
                        .borrow_mut()
                        .synchronized_output = false;
                    (state.on_timeout)()?;
                }
                Ok(())
            }));
            let window = (self.0.window)();
            let handle = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                SYNCHRONIZED_OUTPUT_TIMEOUT_MS,
            )?;
            *self.0.callback.borrow_mut() = Some(callback);
            self.0.timeout.set(Some(handle));
        }
        Ok(())
    }
    pub fn flush(&self) -> Option<RenderRange> {
        self.0.cancel_timer();
        if self.0.buffering.replace(false) {
            Some(self.0.range.get())
        } else {
            None
        }
    }
    pub fn dispose(&self) {
        self.0.cancel_timer();
    }
}
