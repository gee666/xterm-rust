// Copyright (c) 2018-2026 The xterm.js authors. All rights reserved. MIT license.
//! WebglRenderer's context events and physical pixel observer. GPU/model/atlas
//! reconstruction stays with the renderer, invoked before the restoration redraw.
use super::{
    canvas_context::{method, CanvasContext},
    device_pixel_observer::{observe_device_pixel_dimensions, DevicePixelObserver},
};
use crate::{
    dom_listener::DomListener,
    renderer::service_api::{BrowserService, RedrawRequest},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Window;
use xterm_core::{
    event::{Emitter, Event},
    lifecycle::{Disposable, DisposableHandle},
};

const CONTEXT_RESTORATION_TIMEOUT_MS: f64 = 3000.0;
pub type ContextAction = Rc<dyn Fn() -> Result<(), JsValue>>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContextLog {
    Lost,
    Restored,
    NotRestored,
}
/// The actual renderer supplies its live terminal rows, logger, cursor manager,
/// and cache-release/GPU-reinitialization operation. No callback runs under a
/// mutable borrow of lifecycle state. Errors stop the source phase immediately.
/// Callbacks into an enclosing Rc-owned renderer must capture Weak handles so
/// that renderer -> lifecycle -> callback does not form an ownership cycle.
pub struct ContextCallbacks {
    pub rows: Rc<dyn Fn() -> Result<f64, JsValue>>,
    pub log: Rc<dyn Fn(ContextLog) -> Result<(), JsValue>>,
    pub restore: ContextAction,
    pub restart_cursor: ContextAction,
}
struct State {
    context: Rc<CanvasContext>,
    callbacks: ContextCallbacks,
    timeout: RefCell<JsValue>,
    observer: RefCell<Option<Rc<DevicePixelObserver>>>,
    observer_disposed: Cell<bool>,
    window_subscription: RefCell<Option<DisposableHandle>>,
    listeners: RefCell<Vec<Option<DomListener>>>,
    redraw: Emitter<RedrawRequest>,
    context_loss: Emitter<web_sys::Event>,
    disposal_attempted: Cell<bool>,
}
/// Owns actual canvas listeners, the existing device-pixel observer and the
/// source-global restoration timer. Install after other renderer services have
/// been initialized, attach the canvas next, then initialize GPU resources.
/// This component alone is not a renderer or public WebglAddon.
pub struct ContextLifecycle(Rc<State>);
impl ContextLifecycle {
    pub fn new(
        context: Rc<CanvasContext>,
        browser: &dyn BrowserService,
        callbacks: ContextCallbacks,
    ) -> Result<Self, JsValue> {
        let state = Rc::new(State {
            context,
            callbacks,
            timeout: RefCell::new(JsValue::UNDEFINED),
            observer: RefCell::new(None),
            observer_disposed: Cell::new(false),
            window_subscription: RefCell::new(None),
            listeners: RefCell::new(Vec::new()),
            redraw: Emitter::new(),
            context_loss: Emitter::new(),
            disposal_attempted: Cell::new(false),
        });
        let lifecycle = Self(Rc::clone(&state));
        let weak = Rc::downgrade(&state);
        let listener = DomListener::new_reentrant_fallible(
            state.context.canvas().as_ref(),
            "webglcontextlost",
            false,
            None,
            move |event| {
                if let Some(state) = weak.upgrade() {
                    state.lost(event)?;
                }
                Ok(())
            },
        )?;
        state.listeners.borrow_mut().push(Some(listener));
        let weak = Rc::downgrade(&state);
        let listener = DomListener::new_reentrant_fallible(
            state.context.canvas().as_ref(),
            "webglcontextrestored",
            false,
            None,
            move |_| {
                if let Some(state) = weak.upgrade() {
                    state.restored()?;
                }
                Ok(())
            },
        )?;
        state.listeners.borrow_mut().push(Some(listener));
        state.replace_observer(&browser.window())?;
        let weak = Rc::downgrade(&state);
        let subscription = browser.on_window_change().try_subscribe(move |window| {
            if let Some(state) = weak.upgrade() {
                state
                    .replace_observer(window)
                    .map_err(crate::js_error::callback_error)?;
            }
            Ok(())
        });
        *state.window_subscription.borrow_mut() = Some(subscription);
        let weak = Rc::downgrade(&state);
        let document = state.context.main_document();
        let listener = DomListener::new_reentrant_fallible(
            document.as_ref(),
            "mousedown",
            false,
            None,
            move |_| {
                if let Some(state) = weak.upgrade() {
                    (state.callbacks.restart_cursor)()?;
                }
                Ok(())
            },
        )?;
        state.listeners.borrow_mut().push(Some(listener));
        Ok(lifecycle)
    }
    pub fn on_request_redraw(&self) -> Event<RedrawRequest> {
        self.0.redraw.event()
    }
    pub fn on_context_loss(&self) -> Event<web_sys::Event> {
        self.0.context_loss.event()
    }
    pub fn set_device_pixel_dimensions(
        &self,
        width: &JsValue,
        height: &JsValue,
    ) -> Result<(), JsValue> {
        self.0.resize(width, height)
    }
    /// Remove subscriptions and cancel the current restoration timer. The
    /// enclosing renderer disposes layers, detaches the canvas and releases its
    /// atlas afterward. Explicit errors propagate; Drop never retries a
    /// failed explicit cleanup or keeps a dead renderer alive through callbacks.
    pub fn dispose(&self) -> Result<(), JsValue> {
        if self.0.disposal_attempted.replace(true) {
            return Ok(());
        }
        // The observer's MutableDisposable is registered as a field before
        // the constructor registers the context listeners.
        self.0.observer_disposed.set(true);
        let observer = self.0.observer.borrow().clone();
        if let Some(observer) = observer {
            observer.dispose()?;
        }
        self.0.observer.borrow_mut().take();
        self.0.redraw.dispose();
        self.0.context_loss.dispose();
        self.0.remove_listener(0)?;
        self.0.remove_listener(1)?;
        let subscription = self.0.window_subscription.borrow_mut().take();
        if let Some(subscription) = subscription {
            subscription.dispose();
        }
        self.0.remove_listener(2)?;
        self.0.clear_timeout()
    }
}
impl State {
    fn remove_listener(&self, index: usize) -> Result<(), JsValue> {
        let listener = self
            .listeners
            .borrow_mut()
            .get_mut(index)
            .and_then(Option::take);
        if let Some(mut listener) = listener {
            if let Err(error) = listener.dispose() {
                self.listeners.borrow_mut()[index] = Some(listener);
                return Err(error);
            }
        }
        Ok(())
    }
    fn resize(&self, width: &JsValue, height: &JsValue) -> Result<(), JsValue> {
        if self.context.set_device_pixel_dimensions(width, height)? {
            self.redraw(true)?;
        }
        Ok(())
    }
    fn redraw(&self, sync: bool) -> Result<(), JsValue> {
        let end = (self.callbacks.rows)()? - 1.0;
        self.redraw
            .fire(&RedrawRequest {
                start: 0.0,
                end,
                sync,
            })
            .map_err(|error| crate::js_error::to_js_error(&error))
    }
    fn lost(self: &Rc<Self>, event: web_sys::Event) -> Result<(), JsValue> {
        (self.callbacks.log)(ContextLog::Lost)?;
        event.prevent_default();
        let weak = Rc::downgrade(self);
        let callback = Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || {
            if let Some(state) = weak.upgrade() {
                state.timeout.replace(JsValue::UNDEFINED);
                (state.callbacks.log)(ContextLog::NotRestored)?;
                state
                    .context_loss
                    .fire(&event)
                    .map_err(|error| crate::js_error::to_js_error(&error))?;
            }
            Ok(())
        })
        .into_js_value();
        let global = js_sys::global();
        let handle = method(&global, "setTimeout")?.call2(
            &JsValue::UNDEFINED,
            &callback,
            &CONTEXT_RESTORATION_TIMEOUT_MS.into(),
        )?;
        self.timeout.replace(handle);
        Ok(())
    }
    fn clear_timeout(&self) -> Result<(), JsValue> {
        let global = js_sys::global();
        let clear = method(&global, "clearTimeout")?;
        let handle = self.timeout.borrow().clone();
        clear.call1(&JsValue::UNDEFINED, &handle)?;
        self.timeout.replace(JsValue::UNDEFINED);
        Ok(())
    }
    fn restored(&self) -> Result<(), JsValue> {
        (self.callbacks.log)(ContextLog::Restored)?;
        self.clear_timeout()?;
        (self.callbacks.restore)()?;
        self.redraw(false)
    }
    fn replace_observer(self: &Rc<Self>, window: &Window) -> Result<(), JsValue> {
        let weak = Rc::downgrade(self);
        let callback = Closure::<dyn Fn(JsValue, JsValue) -> Result<(), JsValue>>::new(
            move |width, height| {
                if let Some(state) = weak.upgrade() {
                    state.resize(&width, &height)?;
                }
                Ok(())
            },
        )
        .into_js_value();
        // Source evaluates the new observer before MutableDisposable disposes
        // the previous one. Synchronous observe delivery sees that old observer.
        let observer = observe_device_pixel_dimensions(
            self.context.canvas().unchecked_ref(),
            window,
            callback.unchecked_ref(),
        )?;
        if self.observer_disposed.get() {
            // A disposed source MutableDisposable ignores the value and leaks
            // its observer. Rust disconnects the unadopted observer via RAII.
            return Ok(());
        }
        let previous = self.observer.borrow().clone();
        if let Some(previous) = previous {
            previous.dispose()?;
        }
        self.observer.replace(Some(Rc::new(observer)));
        Ok(())
    }
}
impl Drop for ContextLifecycle {
    fn drop(&mut self) {
        if !self.0.disposal_attempted.get() {
            if let Err(error) = self.dispose() {
                crate::write_scheduler::report_unhandled_error(&error);
            }
        }
    }
}
