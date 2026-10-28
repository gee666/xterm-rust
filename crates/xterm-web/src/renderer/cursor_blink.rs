// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{HtmlElement, Window};
const IDLE_TIMEOUT_MS: i32 = 5 * 60 * 1000;
type TimerCallback = Closure<dyn Fn() -> Result<(), JsValue>>;
struct State {
    window: Window,
    row: HtmlElement,
    handle: Cell<Option<i32>>,
    callback: RefCell<Option<TimerCallback>>,
    idle: Cell<bool>,
}
impl State {
    fn clear(&self) {
        if let Some(handle) = self.handle.take() {
            self.window.clear_timeout_with_handle(handle);
        }
        self.callback.borrow_mut().take();
    }
    fn reset(self: &Rc<Self>) -> Result<(), JsValue> {
        self.idle.set(false);
        self.clear();
        let weak = Rc::downgrade(self);
        let callback: TimerCallback = Closure::wrap(Box::new(move || {
            if let Some(state) = weak.upgrade() {
                state.row.class_list().add_1("xterm-cursor-blink-idle")?;
                state.idle.set(true);
                state.handle.set(None);
            }
            Ok(())
        }));
        let handle = self
            .window
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                IDLE_TIMEOUT_MS,
            )?;
        *self.callback.borrow_mut() = Some(callback);
        self.handle.set(Some(handle));
        Ok(())
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.clear();
    }
}
#[derive(Clone)]
pub struct CursorBlinkController(Rc<State>);
impl CursorBlinkController {
    pub fn new(window: Window, row: HtmlElement, focused: bool) -> Result<Self, JsValue> {
        let state = Rc::new(State {
            window,
            row,
            handle: Cell::new(None),
            callback: RefCell::new(None),
            idle: Cell::new(false),
        });
        if focused {
            state.reset()?;
        }
        Ok(Self(state))
    }
    pub fn restart(&self) -> Result<(), JsValue> {
        if self.0.idle.get() {
            self.0
                .row
                .class_list()
                .remove_1("xterm-cursor-blink-idle")?;
        }
        self.0.reset()
    }
    pub fn pause(&self) {
        self.0.idle.set(false);
        self.0.clear();
    }
    pub fn resume(&self) -> Result<(), JsValue> {
        self.0.idle.set(false);
        self.0
            .row
            .class_list()
            .remove_1("xterm-cursor-blink-idle")?;
        self.0.reset()
    }
    pub fn dispose(&self) {
        self.0.clear();
    }
}
