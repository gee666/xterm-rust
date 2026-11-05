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
type FrameCallback = Closure<dyn FnMut(f64) -> Result<(), JsValue>>;
type TimerCallback = Closure<dyn Fn() -> Result<(), JsValue>>;
pub type WindowSource = Rc<dyn Fn() -> Window>;
/// Resolves the current owner without erasing a host getter's thrown value.
pub type FallibleWindowSource = Rc<dyn Fn() -> Result<Window, JsValue>>;

struct BrowserFrameScheduler {
    window: Rc<RefCell<FallibleWindowSource>>,
    callback: CallbackSlot,
}
impl FrameScheduler for BrowserFrameScheduler {
    type Handle = i32;
    type Error = JsValue;
    fn request_frame(&mut self) -> Result<i32, JsValue> {
        Self::request(self.window.borrow().clone(), self.callback.borrow().clone())
    }
    fn cancel_frame(&mut self, handle: i32) -> Result<(), JsValue> {
        let source = self.window.borrow().clone();
        source()?.cancel_animation_frame(handle)
    }
    fn request_frame_shared(scheduler: &RefCell<Self>) -> Result<i32, JsValue> {
        let (source, callback) = {
            let scheduler = scheduler.borrow();
            let source = scheduler.window.borrow().clone();
            let callback = scheduler.callback.borrow().clone();
            (source, callback)
        };
        Self::request(source, callback)
    }
    fn cancel_frame_resolved(
        scheduler: &RefCell<Self>,
        resolve: impl FnOnce() -> Option<i32>,
    ) -> Result<(), JsValue> {
        let source = scheduler.borrow().window.borrow().clone();
        let window = source()?;
        let cancel: Function =
            js_sys::Reflect::get(window.as_ref(), &"cancelAnimationFrame".into())?.unchecked_into();
        let handle = resolve().map_or(JsValue::UNDEFINED, JsValue::from);
        js_sys::Reflect::apply(&cancel, window.as_ref(), &[handle].into_iter().collect())?;
        Ok(())
    }
    fn cleanup_error(&mut self, error: JsValue) {
        web_sys::console::error_1(&error);
    }
}
impl BrowserFrameScheduler {
    fn request(source: FallibleWindowSource, callback: Option<Function>) -> Result<i32, JsValue> {
        let callback =
            callback.ok_or_else(|| JsValue::from_str("animation callback not initialized"))?;
        // Both window resolution and RAF can reenter this debouncer. Only
        // independent handles may survive either host call.
        source()?.request_animation_frame(&callback)
    }
}

/// A real requestAnimationFrame host. It owns the Closure and cancels before
/// releasing it. The closure captures weak core state, never a strong owner.
pub struct BrowserRenderDebouncer {
    inner: RenderDebouncer<BrowserFrameScheduler>,
    window: Rc<RefCell<FallibleWindowSource>>,
    callback: Option<FrameCallback>,
}
impl BrowserRenderDebouncer {
    pub fn new(window: Window, render: impl Fn(RenderRange) + 'static) -> Self {
        Self::try_new(window, move |range| {
            render(range);
            Ok(())
        })
    }
    pub fn try_new(
        window: Window,
        render: impl Fn(RenderRange) -> Result<(), JsValue> + 'static,
    ) -> Self {
        Self::try_new_with_window_source(Rc::new(move || window.clone()), render)
    }
    /// Read the service's current owner directly, including during nested window
    /// change/DPR events before a later window-change listener could run.
    pub fn try_new_with_window_source(
        source: WindowSource,
        render: impl Fn(RenderRange) -> Result<(), JsValue> + 'static,
    ) -> Self {
        Self::try_new_with_fallible_window_source(Rc::new(move || Ok(source())), render)
    }
    /// Resolves the owner on every request and cancellation. Lookup failures are
    /// returned unchanged, after row/callback updates but before frame assignment
    /// or cancellation. The resolver may reenter without a scheduler borrow.
    pub fn try_new_with_fallible_window_source(
        source: FallibleWindowSource,
        render: impl Fn(RenderRange) -> Result<(), JsValue> + 'static,
    ) -> Self {
        let slot: CallbackSlot = Rc::new(RefCell::new(None));
        let window = Rc::new(RefCell::new(source));
        let inner = RenderDebouncer::try_new(
            BrowserFrameScheduler {
                window: Rc::clone(&window),
                callback: slot.clone(),
            },
            move |_, rows| render(rows),
        );
        let dispatch = inner.weak_dispatcher();
        let callback = Closure::wrap(
            Box::new(move |_: f64| dispatch()) as Box<dyn FnMut(f64) -> Result<(), JsValue>>
        );
        *slot.borrow_mut() = Some(callback.as_ref().unchecked_ref::<Function>().clone());
        Self {
            inner,
            window,
            callback: Some(callback),
        }
    }
    /// Request and cancellation use the current CoreBrowserService owner window,
    /// including cancellation of an id originally issued by a previous window.
    pub fn set_window(&self, window: Window) {
        *self.window.borrow_mut() = Rc::new(move || Ok(window.clone()));
    }
    pub fn refresh(&self, start: Option<f64>, end: Option<f64>, count: f64) -> Result<(), JsValue> {
        self.inner.refresh(start, end, count)
    }
    pub fn add_refresh_callback(&self, callback: impl Fn(f64) + 'static) -> Result<i32, JsValue> {
        self.inner
            .add_refresh_callback(move |_, time| callback(time))
    }
    pub fn add_try_refresh_callback(
        &self,
        callback: impl Fn(f64) -> Result<(), JsValue> + 'static,
    ) -> Result<i32, JsValue> {
        self.inner
            .add_try_refresh_callback(move |_, time| callback(time))
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

/// Reusable browser timeout/clock adapter. The caller owns any Rust Closure
/// behind callback and must cancel outstanding jobs before releasing that owner.
#[derive(Clone)]
pub struct BrowserTimerScheduler {
    window: Window,
    performance: Performance,
    callback: CallbackSlot,
}
impl BrowserTimerScheduler {
    pub fn new(window: Window, callback: Function) -> Result<Self, JsValue> {
        let performance = window
            .performance()
            .ok_or_else(|| JsValue::from_str("window.performance unavailable"))?;
        Ok(Self {
            window,
            performance,
            callback: Rc::new(RefCell::new(Some(callback))),
        })
    }
}
impl TimerScheduler for BrowserTimerScheduler {
    type Handle = i32;
    type Error = JsValue;
    fn now(&self) -> f64 {
        self.performance.now()
    }
    fn request_timeout(&mut self, delay: f64) -> Result<i32, JsValue> {
        let callback = self
            .callback
            .borrow()
            .clone()
            .ok_or_else(|| JsValue::from_str("timer callback not initialized"))?;
        // Calling the binding with an i32 would clamp in Rust before WebIDL's
        // conversion. Reflect.apply retains JS number conversion for all delays.
        let set_timeout: Function =
            js_sys::Reflect::get(self.window.as_ref(), &JsValue::from_str("setTimeout"))?
                .dyn_into()?;
        let args = js_sys::Array::new();
        args.push(&callback);
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
        let clear_timeout: Function =
            js_sys::Reflect::get(self.window.as_ref(), &JsValue::from_str("clearTimeout"))?
                .dyn_into()?;
        clear_timeout.call1(self.window.as_ref(), &JsValue::from_f64(f64::from(handle)))?;
        Ok(())
    }
    fn now_shared(scheduler: &RefCell<Self>) -> Result<f64, JsValue> {
        let performance = scheduler.borrow().performance.clone();
        let now: Function =
            js_sys::Reflect::get(performance.as_ref(), &"now".into())?.unchecked_into();
        now.call0(performance.as_ref())?
            .as_f64()
            .ok_or_else(|| JsValue::from_str("performance.now did not return a number"))
    }
    fn request_timeout_shared(scheduler: &RefCell<Self>, delay: f64) -> Result<i32, JsValue> {
        let mut host = scheduler.borrow().clone();
        host.request_timeout(delay)
    }
    fn cancel_timeout_resolved(
        scheduler: &RefCell<Self>,
        resolve: impl FnOnce() -> Option<i32>,
    ) -> Result<(), JsValue> {
        let window = scheduler.borrow().window.clone();
        let cancel: Function =
            js_sys::Reflect::get(window.as_ref(), &"clearTimeout".into())?.unchecked_into();
        let handle = resolve().map_or(JsValue::UNDEFINED, JsValue::from);
        cancel.call1(window.as_ref(), &handle)?;
        Ok(())
    }
    fn cleanup_error(&mut self, error: JsValue) {
        web_sys::console::error_1(&error);
    }
}

pub struct BrowserTimeDebouncer {
    inner: TimeBasedDebouncer<BrowserTimerScheduler>,
    callback: Option<TimerCallback>,
}
impl BrowserTimeDebouncer {
    pub fn new(
        window: Window,
        threshold_ms: f64,
        render: impl Fn(RenderRange) + 'static,
    ) -> Result<Self, JsValue> {
        Self::try_new(window, threshold_ms, move |range| {
            render(range);
            Ok(())
        })
    }
    pub fn try_new(
        window: Window,
        threshold_ms: f64,
        render: impl Fn(RenderRange) -> Result<(), JsValue> + 'static,
    ) -> Result<Self, JsValue> {
        let performance = window
            .performance()
            .ok_or_else(|| JsValue::from_str("window.performance unavailable"))?;
        let slot: CallbackSlot = Rc::new(RefCell::new(None));
        let inner = TimeBasedDebouncer::try_new(
            BrowserTimerScheduler {
                window,
                performance,
                callback: slot.clone(),
            },
            threshold_ms,
            move |_, rows| render(rows),
        );
        let dispatch = inner.weak_dispatcher();
        let callback = Closure::wrap(Box::new(dispatch) as Box<dyn Fn() -> Result<(), JsValue>>);
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
