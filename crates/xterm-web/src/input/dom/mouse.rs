// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
//! DOM ownership for MouseService. The coordinator supplies live engine state.
#[path = "mouse_alt.rs"]
mod alt;
#[path = "mouse_events.rs"]
mod events;
#[path = "mouse_subscriptions.rs"]
mod subscriptions;

use super::InputHost;
use crate::input::{
    mouse_coords::{parse_css_padding, MouseGeometry},
    mouse_policy::{MouseOutput, MousePolicy, MouseRouting},
};
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

#[derive(Clone, Copy, Debug)]
pub struct MouseSnapshot {
    pub routing: MouseRouting,
    pub valid_char_size: bool,
    pub css_cell_width: f64,
    pub css_cell_height: f64,
    pub css_canvas_width: f64,
    pub css_canvas_height: f64,
    pub has_scrollback: bool,
    pub application_cursor: bool,
}

/// All methods observe live state. Implementors must use the terminal's shared
/// unit-payload mouse service, not a second service with independent modes.
pub trait MouseHost {
    fn mouse_service(&self) -> Result<Rc<MouseStateService<()>>, JsValue>;
    fn mouse_snapshot(&self) -> Result<MouseSnapshot, JsValue>;
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
    protocol_error: RefCell<Option<JsValue>>,
}

pub struct MouseBindings {
    state: Rc<State>,
    fixed: Vec<DomListener>,
    subscription: Option<DisposableHandle>,
    option_subscription: Option<DisposableHandle>,
    gesture: Option<DisposableHandle>,
}

impl MouseBindings {
    pub fn new(dom: &BrowserDom, host: Rc<dyn InputHost>) -> Result<Self, JsValue> {
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
            protocol_error: RefCell::new(None),
        });
        let mut bindings = Self {
            state,
            fixed: Vec::new(),
            subscription: None,
            option_subscription: None,
            gesture: None,
        };
        let (protocol, options) = subscriptions::bind(&bindings.state);
        bindings.subscription = Some(protocol);
        bindings.option_subscription = Some(options);
        // Upstream assigns activeProtocol to itself, notifying all subscribers.
        service
            .set_active_protocol(&service.active_protocol())
            .map_err(|failure| {
                bindings
                    .take_protocol_error()
                    .unwrap_or_else(|| error(failure))
            })?;
        events::bind_fixed(&mut bindings)?;
        Ok(bindings)
    }

    /// The coordinator drains this after a protocol-changing engine operation.
    /// If the engine returned MouseStateError::Dispatch, rethrow this original
    /// JS value rather than replacing a callback/getter exception with a string.
    pub fn take_protocol_error(&self) -> Option<JsValue> {
        self.state.protocol_error.borrow_mut().take()
    }

    /// Invoke after mouseEventsRequireAlt changes, as well as protocol changes.
    pub fn sync_options(&self) -> Result<(), JsValue> {
        if self.state.disposed.get() {
            return Err(error("Mouse input is disposed"));
        }
        sync_mode(&self.state)?;
        alt::sync(&self.state)
    }

    pub fn reset(&self) {
        self.state.policy.reset();
    }

    pub fn dispose(&mut self) -> Result<(), JsValue> {
        self.state.disposed.set(true);
        if let Some(subscription) = self.subscription.take() {
            subscription.dispose();
        }
        if let Some(options) = self.option_subscription.take() {
            options.dispose();
        }
        if let Some(gesture) = self.gesture.take() {
            gesture.dispose();
        }
        for listener in &mut self.fixed {
            listener.dispose()?;
        }
        self.fixed.clear();
        clear_dynamic(&self.state)
    }
}

impl Drop for MouseBindings {
    fn drop(&mut self) {
        self.state.disposed.set(true);
        if let Some(subscription) = self.subscription.take() {
            subscription.dispose();
        }
        if let Some(options) = self.option_subscription.take() {
            options.dispose();
        }
        if let Some(gesture) = self.gesture.take() {
            gesture.dispose();
        }
        // Listener fields own their closures and remove built-in listeners on drop.
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
    let require_alt = state.host.mouse_snapshot()?.routing.require_alt;
    if active && !require_alt {
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

fn geometry(state: &State, snapshot: MouseSnapshot) -> Result<MouseGeometry, JsValue> {
    let rect = state.screen.get_bounding_client_rect();
    let document = state
        .screen
        .owner_document()
        .unwrap_or_else(|| state.document.clone());
    let window = document
        .default_view()
        .ok_or_else(|| error("Mouse input requires a document window"))?;
    let style = window
        .get_computed_style(&state.screen)?
        .ok_or_else(|| error("Mouse screen has no computed style"))?;
    Ok(MouseGeometry {
        rect_left: rect.left(),
        rect_top: rect.top(),
        padding_left: parse_css_padding(&style.get_property_value("padding-left")?),
        padding_top: parse_css_padding(&style.get_property_value("padding-top")?),
        valid_char_size: snapshot.valid_char_size,
        cell_width: snapshot.css_cell_width,
        cell_height: snapshot.css_cell_height,
        canvas_width: snapshot.css_canvas_width,
        canvas_height: snapshot.css_canvas_height,
    })
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
                    return callback(&state, event);
                }
            }
            Ok(())
        },
    )
}
