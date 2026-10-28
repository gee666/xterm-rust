// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under MIT. Port of scrollable/globalPointerMoveMonitor.ts.

use crate::dom_listener::DomListener;
use js_sys::Reflect;
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
use wasm_bindgen::{prelude::*, JsCast};

#[wasm_bindgen(inline_js = "
export function monitorSetCapture(target, id) { target.setPointerCapture(id); }
export function monitorReleaseCapture(target, id) { target.releasePointerCapture(id); }
export function monitorWindow(value) {
  if (value?.ownerDocument?.defaultView) return value.ownerDocument.defaultView;
  if (value?.view) return value.view;
  return window;
}
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = monitorSetCapture)]
    fn monitor_set_capture(target: &JsValue, id: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = monitorReleaseCapture)]
    fn monitor_release_capture(target: &JsValue, id: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(catch, js_name = monitorWindow)]
    fn monitor_window(target: &JsValue) -> Result<JsValue, JsValue>;
}
use web_sys::{Element, Event, EventTarget};
use xterm_core::lifecycle::{to_disposable, Disposable, DisposableStore};

pub type PointerMoveCallback = Rc<dyn Fn(Event) -> Result<(), JsValue>>;
pub type PointerStopCallback = Rc<dyn Fn() -> Result<(), JsValue>>;

#[derive(Default)]
struct State {
    hooks: DisposableStore,
    movement: RefCell<Option<PointerMoveCallback>>,
    stop: RefCell<Option<PointerStopCallback>>,
}
impl State {
    fn monitoring(&self) -> bool {
        self.movement.borrow().is_some()
    }
    fn clear(&self) -> Option<PointerStopCallback> {
        if !self.monitoring() {
            return None;
        }
        // Clear the live source Set before resetting callbacks. Pointer-capture
        // release can synchronously reenter and install more hooks to clear.
        self.hooks.clear();
        self.movement.borrow_mut().take();
        self.stop.borrow_mut().take()
    }
    fn stop(&self, invoke: bool) -> Result<(), JsValue> {
        let callback = self.clear();
        if invoke {
            if let Some(callback) = callback {
                callback()?;
            }
        }
        Ok(())
    }
}

/// Clones share one monitor. Callbacks should capture its weak handle when they
/// need to start or stop it themselves, avoiding a monitor-to-callback cycle.
#[derive(Clone, Default)]
pub struct GlobalPointerMoveMonitor {
    state: Rc<State>,
}
#[derive(Clone)]
pub struct WeakPointerMoveMonitor {
    state: Weak<State>,
}
impl WeakPointerMoveMonitor {
    pub fn upgrade(&self) -> Option<GlobalPointerMoveMonitor> {
        self.state
            .upgrade()
            .map(|state| GlobalPointerMoveMonitor { state })
    }
}
impl GlobalPointerMoveMonitor {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn downgrade(&self) -> WeakPointerMoveMonitor {
        WeakPointerMoveMonitor {
            state: Rc::downgrade(&self.state),
        }
    }
    pub fn is_monitoring(&self) -> bool {
        self.state.monitoring()
    }
    pub fn stop_monitoring(&self, invoke_stop_callback: bool) -> Result<(), JsValue> {
        self.state.stop(invoke_stop_callback)
    }
    pub fn dispose(&self) {
        self.state.clear();
        self.state.hooks.dispose();
    }
    pub fn start_monitoring(
        &self,
        element: Element,
        pointer_id: JsValue,
        initial_buttons: JsValue,
        movement: PointerMoveCallback,
        stop: PointerStopCallback,
    ) -> Result<(), JsValue> {
        self.start_monitoring_raw(element.into(), pointer_id, initial_buttons, movement, stop)
    }

    /// The source rereads event.target immediately before this call. Keep that
    /// raw value, including a changed target or a cross-realm Window fallback.
    pub fn start_monitoring_raw(
        &self,
        element: JsValue,
        pointer_id: JsValue,
        initial_buttons: JsValue,
        movement: PointerMoveCallback,
        stop: PointerStopCallback,
    ) -> Result<(), JsValue> {
        if self.is_monitoring() {
            self.state.stop(false)?;
        }
        *self.state.movement.borrow_mut() = Some(movement);
        *self.state.stop.borrow_mut() = Some(stop);
        let source: EventTarget = match monitor_set_capture(&element, &pointer_id) {
            Ok(()) => {
                let captured = element.clone();
                self.state.hooks.add(to_disposable(move || {
                    // Source explicitly ignores an already-released or denied
                    // capture here, including browser release before pointerup.
                    let _ = monitor_release_capture(&captured, &pointer_id);
                }));
                // This is an opaque host object, not a Rust memory cast. Source
                // accepts duck-typed targets; fallible addEventListener performs
                // the actual boundary validation, including cross-realm nodes.
                element.unchecked_into()
            }
            Err(_) => {
                // Capture denial is the upstream path for synthetic events or
                // unavailable active pointers. Listen on that element's window.
                monitor_window(&element)?.unchecked_into()
            }
        };
        let weak = Rc::downgrade(&self.state);
        let listener = DomListener::new_reentrant_fallible(
            &source,
            "pointermove",
            false,
            None,
            move |event| {
                let Some(state) = weak.upgrade() else {
                    return Ok(());
                };
                let buttons = Reflect::get(event.as_ref(), &"buttons".into())?;
                if buttons != initial_buttons {
                    return state.stop(true);
                }
                super::interop::method0(event.as_ref(), "preventDefault")?;
                let callback = state.movement.borrow().clone().ok_or_else(|| {
                    js_sys::TypeError::new("pointer move callback is no longer registered")
                })?;
                callback(event)
            },
        )?;
        store_listener(&self.state.hooks, listener);
        let weak = Rc::downgrade(&self.state);
        let listener =
            DomListener::new_reentrant_fallible(&source, "pointerup", false, None, move |_| {
                weak.upgrade().map_or(Ok(()), |state| state.stop(true))
            })?;
        store_listener(&self.state.hooks, listener);
        Ok(())
    }
}

/// Adapt the existing RAII DOM listener to the source's live disposable Set.
/// Removing a previously registered DOM listener is nonthrowing by the DOM
/// contract. Take it outside the borrow before releasing its captured closure.
pub(crate) fn store_listener(store: &DisposableStore, listener: DomListener) {
    let listener = Rc::new(RefCell::new(Some(listener)));
    store.add(to_disposable(move || {
        let owned = listener.borrow_mut().take();
        drop(owned);
    }));
}
