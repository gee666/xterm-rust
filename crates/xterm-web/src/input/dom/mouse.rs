// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! DOM ownership for MouseService. The coordinator supplies live engine state.
#[path = "mouse_alt.rs"]
mod alt;
#[path = "mouse_events.rs"]
mod events;
#[path = "mouse_geometry.rs"]
mod geometry;
#[path = "mouse_read.rs"]
mod read;
#[path = "mouse_report.rs"]
mod report;
#[path = "mouse_subscriptions.rs"]
mod subscriptions;
#[path = "mouse_touch.rs"]
mod touch;
#[path = "mouse_values.rs"]
mod values;
use geometry::report_coords;
pub use read::MouseReadHost;

use super::InputHost;
use crate::input::mouse_policy::{MouseOutput, MousePolicy};
use crate::{dom::BrowserDom, dom_listener::DomListener};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, HtmlElement, MouseEvent, WheelEvent};
use xterm_core::{
    lifecycle::DisposableHandle,
    mouse_state::{MouseEventTypes, MouseStateService},
};

const ENABLE_MOUSE_EVENTS: &str = "enable-mouse-events";

/// All methods observe live state. Implementors must use the terminal's shared
/// unit-payload mouse service, not a second service with independent modes.
pub trait MouseHost: MouseReadHost {
    fn mouse_service(&self) -> Result<Rc<MouseStateService<()>>, JsValue>;
    fn mouse_protocol_log(&self, events: MouseEventTypes) -> Result<(), JsValue>;
    fn mouse_option_changes(&self) -> xterm_core::event::Event<&'static str>;
    fn allow_custom_wheel(&self, event: &WheelEvent) -> Result<bool, JsValue>;
    fn mouse_focus(&self) -> Result<(), JsValue>;
    fn should_force_selection(&self, event: &MouseEvent) -> Result<bool, JsValue>;
    fn set_mouse_selection_enabled(&self, enabled: bool) -> Result<(), JsValue>;
    fn mouse_output(&self, output: MouseOutput) -> Result<(), JsValue>;
    fn mouse_touch_scroll(&self, translation_y: f64) -> Result<(), JsValue>;
    fn mouse_gesture_target(&self, screen: &HtmlElement) -> Result<DisposableHandle, JsValue>;
}

#[derive(Default)]
struct Dynamic {
    move_listener: Option<DomListener>,
    wheel_listener: Option<DomListener>,
    up_listener: Option<DomListener>,
    drag_listener: Option<DomListener>,
    alt_listeners: Vec<DomListener>,
}

struct State {
    element: HtmlElement,
    screen: HtmlElement,
    document: Document,
    host: Rc<dyn InputHost>,
    service: Rc<MouseStateService<()>>,
    policy: MousePolicy,
    requested: Cell<MouseEventTypes>,
    dynamic: RefCell<Dynamic>,
    disposed: Cell<bool>,
}

pub struct MouseBindings {
    state: Rc<State>,
    fixed: RefCell<Vec<DomListener>>,
    subscription: RefCell<Option<DisposableHandle>>,
    option_subscription: RefCell<Option<DisposableHandle>>,
    gesture: RefCell<Option<DisposableHandle>>,
}

impl MouseBindings {
    pub fn new(dom: &BrowserDom, host: Rc<dyn InputHost>) -> Result<Self, JsValue> {
        let _receiver = host.retain_host()?;
        let service = host.mouse_service()?;
        let state = Rc::new(State {
            element: dom.element.clone(),
            screen: dom.screen.clone(),
            document: dom.document.clone(),
            host,
            service: Rc::clone(&service),
            policy: MousePolicy::new(),
            requested: Cell::new(MouseEventTypes::NONE),
            dynamic: RefCell::new(Dynamic::default()),
            disposed: Cell::new(false),
        });
        let bindings = Self {
            state,
            fixed: RefCell::new(Vec::new()),
            subscription: RefCell::new(None),
            option_subscription: RefCell::new(None),
            gesture: RefCell::new(None),
        };
        let (protocol, options) = subscriptions::bind(&bindings.state);
        *bindings.subscription.borrow_mut() = Some(protocol);
        *bindings.option_subscription.borrow_mut() = Some(options);
        // Upstream assigns activeProtocol to itself, notifying all subscribers.
        service
            .set_active_protocol(&service.active_protocol())
            .map_err(|failure| crate::js_error::to_js_error(&failure))?;
        events::bind_fixed(&bindings)?;
        Ok(bindings)
    }

    pub fn reset(&self) {
        self.state.policy.reset();
    }

    pub fn dispose(&self) -> Result<(), JsValue> {
        self.state.disposed.set(true);
        let subscription = self.subscription.borrow_mut().take();
        if let Some(subscription) = subscription {
            subscription.dispose();
        }
        let options = self.option_subscription.borrow_mut().take();
        if let Some(options) = options {
            options.dispose();
        }
        let gesture = self.gesture.borrow_mut().take();
        if let Some(gesture) = gesture {
            gesture.dispose();
        }
        let mut listeners = self.fixed.replace(Vec::new());
        for listener in &mut listeners {
            listener.dispose()?;
        }
        clear_dynamic(&self.state)
    }
}

impl Drop for MouseBindings {
    fn drop(&mut self) {
        // Every listener was registered against a validated built-in EventTarget.
        // Native removal cannot fail; explicit dispose propagates overridden-host
        // exceptions while Drop is the same final safeguard as DomListener.
        self.dispose()
            .expect("validated native mouse listener cleanup cannot fail");
    }
}

fn error(value: impl ToString) -> JsValue {
    js_sys::Error::new(&value.to_string()).into()
}

fn clear(listener: Option<DomListener>) -> Result<(), JsValue> {
    if let Some(mut listener) = listener {
        listener.dispose()?;
    }
    Ok(())
}

fn clear_globals(state: &State) -> Result<(), JsValue> {
    let (up, drag) = {
        let mut dynamic = state.dynamic.borrow_mut();
        (dynamic.up_listener.take(), dynamic.drag_listener.take())
    };
    clear(up)?;
    clear(drag)
}

fn clear_dynamic(state: &State) -> Result<(), JsValue> {
    clear_globals(state)?;
    let (movement, wheel, mut alt) = {
        let mut dynamic = state.dynamic.borrow_mut();
        (
            dynamic.move_listener.take(),
            dynamic.wheel_listener.take(),
            std::mem::take(&mut dynamic.alt_listeners),
        )
    };
    clear(movement)?;
    clear(wheel)?;
    for listener in &mut alt {
        listener.dispose()?;
    }
    Ok(())
}

fn sync_mode(state: &State) -> Result<(), JsValue> {
    let active = state.service.are_mouse_events_active();
    if active && !state.host.mouse_require_alt()? {
        state.element.class_list().add_1(ENABLE_MOUSE_EVENTS)?;
        state.host.set_mouse_selection_enabled(false)?;
    } else {
        state.element.class_list().remove_1(ENABLE_MOUSE_EVENTS)?;
        state.host.set_mouse_selection_enabled(true)?;
    }
    Ok(())
}

fn sync_protocol(state: &Rc<State>, requested: MouseEventTypes) -> Result<(), JsValue> {
    state.host.mouse_protocol_log(requested)?;
    sync_mode(state)?;
    alt::sync(state)?;
    state.requested.set(requested);
    let old_move = state.dynamic.borrow_mut().move_listener.take();
    if requested.0 & MouseEventTypes::MOVE.0 == 0 {
        clear(old_move)?;
    } else {
        let listener = match old_move {
            Some(listener) => listener,
            None => listen(
                state,
                state.element.as_ref(),
                "mousemove",
                false,
                events::movement,
            )?,
        };
        state.dynamic.borrow_mut().move_listener = Some(listener);
    }
    let old_wheel = state.dynamic.borrow_mut().wheel_listener.take();
    if requested.0 & MouseEventTypes::WHEEL.0 == 0 {
        clear(old_wheel)?;
    } else {
        let listener = match old_wheel {
            Some(listener) => listener,
            None => listen(
                state,
                state.element.as_ref(),
                "wheel",
                true,
                events::active_wheel,
            )?,
        };
        state.dynamic.borrow_mut().wheel_listener = Some(listener);
    }
    if requested.0 & MouseEventTypes::UP.0 == 0 {
        let listener = state.dynamic.borrow_mut().up_listener.take();
        clear(listener)?;
    }
    if requested.0 & MouseEventTypes::DRAG.0 == 0 {
        let listener = state.dynamic.borrow_mut().drag_listener.take();
        clear(listener)?;
    }
    Ok(())
}

fn listen(
    state: &Rc<State>,
    target: &web_sys::EventTarget,
    name: &str,
    nonpassive: bool,
    callback: impl Fn(&Rc<State>, web_sys::Event) -> Result<(), JsValue> + 'static,
) -> Result<DomListener, JsValue> {
    let weak = Rc::downgrade(state);
    DomListener::new_reentrant_fallible(
        target,
        name,
        false,
        nonpassive.then_some(false),
        move |event| {
            if let Some(state) = weak.upgrade() {
                if !state.disposed.get() {
                    let _receiver = state.host.retain_host()?;
                    return callback(&state, event);
                }
            }
            Ok(())
        },
    )
}
