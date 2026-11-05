// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! Browser timer state from addon-webgl/src/CursorBlinkStateManager.ts.
//! The supplied service is read live. No global window or timer mirror is used.

use crate::scroll::interop::{clock, get, method1, method2, subtract};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

#[path = "cursor_blink_timers.rs"]
mod timers;

const BLINK_INTERVAL_MS: f64 = 600.0;
const IDLE_TIMEOUT_MS: f64 = 5.0 * 60.0 * 1000.0;

#[wasm_bindgen(inline_js = "export function blinkPositive(value) { return value > 0; }")]
extern "C" {
    #[wasm_bindgen(catch, js_name = blinkPositive)]
    fn positive(value: &JsValue) -> Result<bool, JsValue>;
}

fn call1(method: &JsValue, receiver: &JsValue, value: &JsValue) -> Result<(), JsValue> {
    let args = js_sys::Array::of1(value);
    js_sys::Reflect::apply(method.unchecked_ref(), receiver, &args)?;
    Ok(())
}

#[derive(Clone, Copy)]
enum Callback {
    Render,
    Start,
    Interval,
    Idle,
}

struct State {
    service: JsValue,
    render: Rc<dyn Fn() -> Result<(), JsValue>>,
    visible: Cell<bool>,
    idle_paused: Cell<bool>,
    frame: RefCell<JsValue>,
    start: RefCell<JsValue>,
    interval: RefCell<JsValue>,
    idle: RefCell<JsValue>,
    restarted: RefCell<JsValue>,
}

/// Clones share one manager. Explicit operations preserve source errors and may
/// reenter. The last Rust owner cancels remaining timers; late browser callbacks
/// hold weak state and become inert after that owner goes away.
#[derive(Clone)]
pub struct CursorBlinkStateManager(Rc<State>);

impl CursorBlinkStateManager {
    /// `service` provides live `isFocused` and `window` properties. Timer methods
    /// are invoked with the window receiver. `render` is a Rust callable rather
    /// than a JavaScript manager method, so there is no JS manager receiver.
    pub fn new(
        render: Rc<dyn Fn() -> Result<(), JsValue>>,
        service: JsValue,
    ) -> Result<Self, JsValue> {
        let state = Rc::new(State {
            service,
            render,
            visible: Cell::new(true),
            idle_paused: Cell::new(false),
            frame: RefCell::new(JsValue::UNDEFINED),
            start: RefCell::new(JsValue::UNDEFINED),
            interval: RefCell::new(JsValue::UNDEFINED),
            idle: RefCell::new(JsValue::UNDEFINED),
            restarted: RefCell::new(JsValue::UNDEFINED),
        });
        if get(&state.service, "isFocused")?.is_truthy() {
            state.restart_interval(BLINK_INTERVAL_MS.into())?;
            state.reset_idle()?;
        }
        Ok(Self(state))
    }

    pub fn is_cursor_visible(&self) -> bool {
        self.0.visible.get()
    }

    pub fn is_paused(&self) -> bool {
        self.0.is_paused()
    }

    /// Explicit disposal does not mark the manager permanently closed. Source
    /// permits subsequent resume and does not reject already queued callbacks.
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.0.clear()
    }

    pub fn restart_blink_animation(&self) -> Result<(), JsValue> {
        self.0.restart()
    }

    pub fn pause(&self) -> Result<(), JsValue> {
        self.0.pause()
    }

    pub fn resume(&self) -> Result<(), JsValue> {
        self.0.pause()?;
        self.0.restarted.replace(JsValue::UNDEFINED);
        self.0.restart_interval(BLINK_INTERVAL_MS.into())?;
        self.0.reset_idle()?;
        self.0.restart()
    }
}

impl State {
    fn is_paused(&self) -> bool {
        !self.start.borrow().is_truthy() && !self.interval.borrow().is_truthy()
    }

    fn clear_handle(&self, slot: &RefCell<JsValue>, method: &str) -> Result<(), JsValue> {
        if slot.borrow().is_truthy() {
            // Source looks up the window before reading the argument. A getter
            // can reenter and replace the handle before that second read.
            let window = get(&self.service, "window")?;
            let method = get(&window, method)?;
            let handle = slot.borrow().clone();
            call1(&method, &window, &handle)?;
            slot.replace(JsValue::UNDEFINED);
        }
        Ok(())
    }

    fn clear(&self) -> Result<(), JsValue> {
        self.clear_handle(&self.interval, "clearInterval")?;
        self.clear_handle(&self.start, "clearTimeout")?;
        self.clear_handle(&self.frame, "cancelAnimationFrame")?;
        self.clear_handle(&self.idle, "clearTimeout")
    }

    fn pause(&self) -> Result<(), JsValue> {
        self.visible.set(true);
        self.idle_paused.set(false);
        self.clear()
    }

    fn restart(self: &Rc<Self>) -> Result<(), JsValue> {
        if self.idle_paused.get() {
            self.reset_idle()?;
        }
        if self.is_paused() {
            return Ok(());
        }
        self.restarted.replace(clock()?);
        self.visible.set(true);
        if !self.frame.borrow().is_truthy() {
            self.request_frame()?;
        }
        Ok(())
    }

    fn request_frame(self: &Rc<Self>) -> Result<(), JsValue> {
        let window = get(&self.service, "window")?;
        let callback = self.callback(Callback::Render);
        self.frame
            .replace(method1(&window, "requestAnimationFrame", &callback)?);
        Ok(())
    }

    fn remaining(&self) -> Result<JsValue, JsValue> {
        let now = clock()?;
        let restarted = self.restarted.borrow().clone();
        let elapsed = subtract(&now, &restarted)?;
        let remaining = subtract(&BLINK_INTERVAL_MS.into(), &elapsed)?;
        self.restarted.replace(JsValue::UNDEFINED);
        Ok(remaining)
    }
}

impl Drop for State {
    fn drop(&mut self) {
        if let Err(error) = self.clear() {
            crate::write_scheduler::report_unhandled_error(&error);
        }
    }
}
