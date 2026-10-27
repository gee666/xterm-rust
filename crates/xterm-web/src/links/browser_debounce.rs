// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::{
    debounce::{FrameScheduler, RenderDebouncer, RenderRange},
    time_debounce::{TimeBasedDebouncer, TimerScheduler},
};
use js_sys::Function;
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{Performance, Window};

type CallbackSlot = Rc<RefCell<Option<Function>>>;

struct BrowserFrameScheduler {
    window: Window,
    callback: CallbackSlot,
}
impl FrameScheduler for BrowserFrameScheduler {
    type Handle = i32;
    type Error = JsValue;
    fn request_frame(&mut self) -> Result<i32, JsValue> {
        let callback = self.callback.borrow();
        let callback = callback
            .as_ref()
            .ok_or_else(|| JsValue::from_str("animation callback not initialized"))?;
        self.window.request_animation_frame(callback)
    }
    fn cancel_frame(&mut self, handle: i32) -> Result<(), JsValue> {
        self.window.cancel_animation_frame(handle)
    }
    fn cleanup_error(&mut self, error: JsValue) {
        web_sys::console::error_1(&error);
    }
}

/// A real requestAnimationFrame host. It owns the Closure and cancels before
/// releasing it. The closure captures weak core state, never a strong owner.
pub struct BrowserRenderDebouncer {
    inner: RenderDebouncer<BrowserFrameScheduler>,
    callback: Option<Closure<dyn FnMut(f64)>>,
}
impl BrowserRenderDebouncer {
    pub fn new(window: Window, render: impl Fn(RenderRange) + 'static) -> Self {
        let slot: CallbackSlot = Rc::new(RefCell::new(None));
        let inner = RenderDebouncer::new(
            BrowserFrameScheduler {
                window,
                callback: slot.clone(),
            },
            move |_, rows| render(rows),
        );
        let dispatch = inner.weak_dispatcher();
        let callback = Closure::wrap(Box::new(move |_: f64| dispatch()) as Box<dyn FnMut(f64)>);
        *slot.borrow_mut() = Some(callback.as_ref().unchecked_ref::<Function>().clone());
        Self {
            inner,
            callback: Some(callback),
        }
    }
    pub fn refresh(&self, start: Option<f64>, end: Option<f64>, count: f64) -> Result<(), JsValue> {
        self.inner.refresh(start, end, count)
    }
    pub fn add_refresh_callback(&self, callback: impl Fn(f64) + 'static) -> Result<i32, JsValue> {
        self.inner
            .add_refresh_callback(move |_, time| callback(time))
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.inner.dispose()
    }
}
impl Drop for BrowserRenderDebouncer {
    fn drop(&mut self) {
        if let Err(error) = self.inner.dispose() {
            web_sys::console::error_1(&error);
        }
        self.callback.take();
    }
}

struct BrowserTimerScheduler {
    window: Window,
    performance: Performance,
    callback: CallbackSlot,
}
impl TimerScheduler for BrowserTimerScheduler {
    type Handle = i32;
    type Error = JsValue;
    fn now(&self) -> f64 {
        self.performance.now()
    }
    fn request_timeout(&mut self, delay: f64) -> Result<i32, JsValue> {
        let callback = self.callback.borrow();
        let callback = callback
            .as_ref()
            .ok_or_else(|| JsValue::from_str("timer callback not initialized"))?;
        // Calling the binding with an i32 would clamp in Rust before WebIDL's
        // conversion. Reflect.apply retains JS number conversion for all delays.
        let set_timeout: Function =
            js_sys::Reflect::get(self.window.as_ref(), &JsValue::from_str("setTimeout"))?
                .dyn_into()?;
        let args = js_sys::Array::new();
        args.push(callback);
        args.push(&JsValue::from_f64(delay));
        let handle = set_timeout
            .apply(self.window.as_ref(), &args)?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("setTimeout did not return a numeric identifier"))?;
        if handle.fract() != 0.0 || handle < 1.0 || handle > f64::from(i32::MAX) {
            return Err(JsValue::from_str("invalid browser timeout identifier"));
        }
        Ok(handle as i32)
    }
    fn cancel_timeout(&mut self, handle: i32) -> Result<(), JsValue> {
        self.window.clear_timeout_with_handle(handle);
        Ok(())
    }
    fn cleanup_error(&mut self, error: JsValue) {
        web_sys::console::error_1(&error);
    }
}

pub struct BrowserTimeDebouncer {
    inner: TimeBasedDebouncer<BrowserTimerScheduler>,
    callback: Option<Closure<dyn FnMut()>>,
}
impl BrowserTimeDebouncer {
    pub fn new(
        window: Window,
        threshold_ms: f64,
        render: impl Fn(RenderRange) + 'static,
    ) -> Result<Self, JsValue> {
        let performance = window
            .performance()
            .ok_or_else(|| JsValue::from_str("window.performance unavailable"))?;
        let slot: CallbackSlot = Rc::new(RefCell::new(None));
        let inner = TimeBasedDebouncer::new(
            BrowserTimerScheduler {
                window,
                performance,
                callback: slot.clone(),
            },
            threshold_ms,
            move |_, rows| render(rows),
        );
        let dispatch = inner.weak_dispatcher();
        let callback = Closure::wrap(Box::new(dispatch) as Box<dyn FnMut()>);
        *slot.borrow_mut() = Some(callback.as_ref().unchecked_ref::<Function>().clone());
        Ok(Self {
            inner,
            callback: Some(callback),
        })
    }
    pub fn refresh(&self, start: Option<f64>, end: Option<f64>, count: f64) -> Result<(), JsValue> {
        self.inner.refresh(start, end, count)
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.inner.dispose()
    }
}
impl Drop for BrowserTimeDebouncer {
    fn drop(&mut self) {
        if let Err(error) = self.inner.dispose() {
            web_sys::console::error_1(&error);
        }
        self.callback.take();
    }
}
