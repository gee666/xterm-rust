// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Window;
type TimerCallback = Closure<dyn Fn() -> Result<(), JsValue>>;
struct State {
    window: Window,
    duration: Cell<f64>,
    handle: Cell<Option<i32>>,
    callback: RefCell<Option<TimerCallback>>,
    blink_on: Cell<bool>,
    needs: Cell<bool>,
    visible: Cell<bool>,
    render: Rc<dyn Fn() -> Result<(), JsValue>>,
}
impl State {
    fn clear(&self) {
        if let Some(handle) = self.handle.take() {
            self.window.clear_interval_with_handle(handle);
        }
        self.callback.borrow_mut().take();
    }
    fn update(self: &Rc<Self>) -> Result<(), JsValue> {
        if self.duration.get() > 0.0 && self.needs.get() && self.visible.get() {
            if self.handle.get().is_some() {
                return Ok(());
            }
            let was_on = self.blink_on.replace(true);
            let weak = Rc::downgrade(self);
            let callback: TimerCallback = Closure::wrap(Box::new(move || {
                if let Some(state) = weak.upgrade() {
                    state.blink_on.set(!state.blink_on.get());
                    (state.render)()?;
                }
                Ok(())
            }));
            let handle = self
                .window
                .set_interval_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    to_int32(self.duration.get()),
                )?;
            *self.callback.borrow_mut() = Some(callback);
            self.handle.set(Some(handle));
            if !was_on {
                (self.render)()?;
            }
            return Ok(());
        }
        self.clear();
        if !self.blink_on.replace(true) {
            (self.render)()?;
        }
        Ok(())
    }
}
impl Drop for State {
    fn drop(&mut self) {
        self.clear();
    }
}
/// Own the source blink interval; visibility and actual blinking rows gate it.
#[derive(Clone)]
pub struct TextBlinkController(Rc<State>);
impl TextBlinkController {
    pub fn new(
        window: Window,
        duration: f64,
        render: impl Fn() -> Result<(), JsValue> + 'static,
    ) -> Self {
        Self(Rc::new(State {
            window,
            duration: Cell::new(duration),
            handle: Cell::new(None),
            callback: RefCell::new(None),
            blink_on: Cell::new(true),
            needs: Cell::new(false),
            visible: Cell::new(true),
            render: Rc::new(render),
        }))
    }
    pub fn is_blink_on(&self) -> bool {
        self.0.blink_on.get()
    }
    pub fn set_needs_blink(&self, needs: bool) -> Result<(), JsValue> {
        if self.0.needs.replace(needs) == needs {
            return Ok(());
        }
        self.0.update()
    }
    pub fn set_visible(&self, visible: bool) -> Result<(), JsValue> {
        if self.0.visible.replace(visible) == visible {
            return Ok(());
        }
        self.0.update()
    }
    pub fn set_duration(&self, duration: f64) -> Result<(), JsValue> {
        if self.0.duration.get() == duration {
            return Ok(());
        }
        self.0.duration.set(duration);
        self.0.clear();
        self.0.update()
    }
    pub fn dispose(&self) {
        self.0.clear();
    }
}
fn to_int32(value: f64) -> i32 {
    if !value.is_finite() || value == 0.0 {
        0
    } else {
        value.trunc().rem_euclid(4294967296.0) as u32 as i32
    }
}
