// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under MIT. Port of scrollable/scrollbarArrow.ts.
use super::{
    interop::{event_window, get, method0},
    pointer_monitor::{store_listener, GlobalPointerMoveMonitor},
};
use crate::{dom_listener::DomListener, links::window_interval::WindowIntervalTimer};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Event, HtmlElement, Window};
use xterm_core::{
    lifecycle::{Disposable, DisposableStore},
    options::OptionValue as V,
};

#[path = "arrow_layout.rs"]
mod layout;
#[path = "arrow_timeout.rs"]
mod timeout;
const REPEAT_DELAY_MS: f64 = 200.0;
const REPEAT_INTERVAL_MS: f64 = 1000.0 / 24.0;

pub struct ArrowOptions {
    pub class_name: V,
    pub bg_width: V,
    pub bg_height: V,
    pub top: Option<V>,
    pub left: Option<V>,
    pub bottom: Option<V>,
    pub right: Option<V>,
    pub handle_activate: Rc<dyn Fn() -> Result<(), JsValue>>,
}
struct State {
    window: Window,
    activate: Rc<dyn Fn() -> Result<(), JsValue>>,
    monitor: GlobalPointerMoveMonitor,
    listeners: DisposableStore,
    repeat: WindowIntervalTimer,
    delayed_repeat: timeout::ArrowTimeout,
    disposed: Cell<bool>,
}
impl State {
    fn pointer_down(self: &Rc<Self>, event: Event) -> Result<(), JsValue> {
        if get(event.as_ref(), "target")?.is_falsy() {
            return Ok(());
        }
        if get(event.as_ref(), "target")?
            .dyn_ref::<Element>()
            .is_none()
        {
            return Ok(());
        }
        // Keep the event, not its view snapshot. Source resolves the repeat
        // interval's target window only when the 200 ms timeout fires.
        let pending_event = event.clone();
        let weak = Rc::downgrade(self);
        let schedule: Rc<dyn Fn() -> Result<(), JsValue>> = Rc::new(move || {
            let Some(state) = weak.upgrade() else {
                return Ok(());
            };
            let target =
                event_window(pending_event.as_ref(), &state.window)?.unchecked_into::<Window>();
            let repeat = Rc::downgrade(&state);
            state.repeat.cancel_and_set(
                move |_| {
                    if let Some(state) = repeat.upgrade() {
                        (state.activate)()?;
                    }
                    Ok(())
                },
                REPEAT_INTERVAL_MS,
                Some(&target),
            )
        });
        (self.activate)()?;
        self.repeat.cancel()?;
        self.delayed_repeat
            .cancel_and_set(schedule, REPEAT_DELAY_MS)?;
        let target = get(event.as_ref(), "target")?;
        let pointer_id = get(event.as_ref(), "pointerId")?;
        let buttons = get(event.as_ref(), "buttons")?;
        let weak = Rc::downgrade(self);
        self.monitor.start_monitoring_raw(
            target,
            pointer_id,
            buttons,
            // Source monitors release/buttons only; movement has no action.
            Rc::new(|_| Ok(())),
            Rc::new(move || {
                if let Some(state) = weak.upgrade() {
                    state.repeat.cancel()?;
                    state.delayed_repeat.cancel()?;
                }
                Ok(())
            }),
        )?;
        method0(event.as_ref(), "preventDefault")?;
        Ok(())
    }
    fn dispose(&self) -> Result<(), JsValue> {
        if self.disposed.replace(true) {
            return Ok(());
        }
        // Source registration order is monitor, both DOM hooks, interval, timeout.
        self.monitor.dispose();
        self.listeners.dispose();
        self.repeat.dispose()?;
        self.delayed_repeat.dispose()
    }
}
impl Drop for State {
    fn drop(&mut self) {
        if let Err(error) = self.dispose() {
            web_sys::console::error_1(&error);
        }
    }
}

#[derive(Clone)]
pub struct ScrollbarArrow {
    pub bg_dom_node: HtmlElement,
    pub dom_node: HtmlElement,
    state: Rc<State>,
}
impl ScrollbarArrow {
    pub fn new(window: Window, options: ArrowOptions) -> Result<Self, JsValue> {
        let (bg_dom_node, dom_node) = layout::nodes(&options)?;
        let state = Rc::new(State {
            repeat: WindowIntervalTimer::new(None, window.clone()),
            window,
            activate: options.handle_activate,
            monitor: GlobalPointerMoveMonitor::new(),
            listeners: DisposableStore::new(),
            delayed_repeat: timeout::ArrowTimeout::new(),
            disposed: Cell::new(false),
        });
        for node in [&bg_dom_node, &dom_node] {
            let weak = Rc::downgrade(&state);
            let listener = DomListener::new_reentrant_fallible(
                node.as_ref(),
                "pointerdown",
                false,
                None,
                move |event| {
                    if let Some(state) = weak.upgrade() {
                        state.pointer_down(event)?;
                    }
                    Ok(())
                },
            )?;
            store_listener(&state.listeners, listener);
        }
        Ok(Self {
            bg_dom_node,
            dom_node,
            state,
        })
    }
    pub fn dispose(&self) -> Result<(), JsValue> {
        self.state.dispose()
    }
}
