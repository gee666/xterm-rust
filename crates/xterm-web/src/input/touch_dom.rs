// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Document-owned recognizer for browser/scrollable/touch.ts.
#[path = "touch_animation.rs"]
mod animation;
#[path = "touch_events.rs"]
mod events;

use super::touch::{Inertia, TapCount, TouchData};
use crate::{
    dom_listener::DomListener,
    links::animation_queue::{AnimationItem, WindowAnimationQueue, WindowAnimationQueues},
};
use std::{
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Document, Event, EventTarget, HtmlElement, Node};
use xterm_core::lifecycle::{to_disposable, Disposable, DisposableHandle};

const TAP: &str = "-xterm-gesturetap";
const CHANGE: &str = "-xterm-gesturechange";
const START: &str = "-xterm-gesturestart";
const END: &str = "-xterm-gesturesend";
const CONTEXT_MENU: &str = "-xterm-gesturecontextmenu";
thread_local! {
    // Source Disposable.None has shared immutable identity. This stores no
    // gesture state and cannot couple document registries or active touches.
    static NO_TOUCH: DisposableHandle = Rc::new(xterm_core::lifecycle::NoopDisposable);
}

type TouchRecord = Rc<RefCell<TouchData<Option<EventTarget>>>>;
struct Target {
    id: u64,
    element: HtmlElement,
}

/// Share one context per document through RuntimeContext. Listener closures keep
/// only Weak context references, and each registration token removes its own node.
pub struct GestureContext {
    document: Document,
    touch_device: bool,
    targets: RefCell<Vec<Target>>,
    ignores: RefCell<Vec<Target>>,
    active: RefCell<HashMap<i32, TouchRecord>>,
    next_id: Cell<u64>,
    tap_count: RefCell<TapCount>,
    dispatched: Cell<bool>,
    listeners: RefCell<Vec<DomListener>>,
    animation_queue: WindowAnimationQueue,
    frame: RefCell<Option<AnimationItem>>,
}

impl GestureContext {
    pub fn new(document: Document) -> Result<Rc<Self>, JsValue> {
        let window = document
            .default_view()
            .ok_or_else(|| error("Gesture document has no window"))?;
        let queue = WindowAnimationQueues::default().for_window(&window);
        Self::new_with_queue(document, queue)
    }

    /// RuntimeContext supplies its shared per-window queue so inertia coalesces
    /// with other Dom.scheduleAtNextAnimationFrame users in source order.
    pub fn new_with_queue(
        document: Document,
        animation_queue: WindowAnimationQueue,
    ) -> Result<Rc<Self>, JsValue> {
        let window = document
            .default_view()
            .ok_or_else(|| error("Gesture document has no window"))?;
        let touch_device = js_sys::Reflect::has(&window, &"ontouchstart".into())?
            || window.navigator().max_touch_points() > 0;
        let context = Rc::new(Self {
            document,
            touch_device,
            targets: RefCell::new(Vec::new()),
            ignores: RefCell::new(Vec::new()),
            active: RefCell::new(HashMap::new()),
            next_id: Cell::new(0),
            tap_count: RefCell::new(TapCount::default()),
            dispatched: Cell::new(false),
            listeners: RefCell::new(Vec::new()),
            animation_queue,
            frame: RefCell::new(None),
        });
        if touch_device {
            for (name, nonpassive, handler) in [
                (
                    "touchstart",
                    true,
                    events::start as fn(&Rc<Self>, Event) -> Result<(), JsValue>,
                ),
                ("touchend", false, events::end),
                ("touchmove", true, events::movement),
            ] {
                let weak = Rc::downgrade(&context);
                let callback = move |event| {
                    weak.upgrade()
                        .map_or(Ok(()), |context| handler(&context, event))
                };
                let listener = DomListener::new_reentrant_fallible(
                    context.document.as_ref(),
                    name,
                    false,
                    nonpassive.then_some(false),
                    callback,
                )?;
                context.listeners.borrow_mut().push(listener);
            }
        }
        Ok(context)
    }

    pub fn add_target(self: &Rc<Self>, element: &HtmlElement) -> Result<DisposableHandle, JsValue> {
        self.register(element, false)
    }
    pub fn ignore_target(
        self: &Rc<Self>,
        element: &HtmlElement,
    ) -> Result<DisposableHandle, JsValue> {
        self.register(element, true)
    }
    pub fn is_touch_device(&self) -> bool {
        self.touch_device
    }

    fn register(
        self: &Rc<Self>,
        element: &HtmlElement,
        ignored: bool,
    ) -> Result<DisposableHandle, JsValue> {
        if !self.touch_device {
            return Ok(NO_TOUCH.with(Rc::clone));
        }
        let id = self
            .next_id
            .get()
            .checked_add(1)
            .ok_or_else(|| error("Gesture registration limit exceeded"))?;
        self.next_id.set(id);
        let entries = if ignored {
            &self.ignores
        } else {
            &self.targets
        };
        entries.borrow_mut().push(Target {
            id,
            element: element.clone(),
        });
        let weak = Rc::downgrade(self);
        Ok(to_disposable(move || {
            if let Some(context) = weak.upgrade() {
                let entries = if ignored {
                    &context.ignores
                } else {
                    &context.targets
                };
                entries.borrow_mut().retain(|entry| entry.id != id);
            }
        }))
    }

    pub fn dispose(&self) -> Result<(), JsValue> {
        self.cancel_frame();
        let listeners = std::mem::take(&mut *self.listeners.borrow_mut());
        for mut listener in listeners {
            listener.dispose()?;
        }
        Ok(())
    }

    fn cancel_frame(&self) {
        let frame = self.frame.borrow_mut().take();
        if let Some(frame) = frame {
            frame.dispose();
        }
    }

    fn event(&self, kind: &str, initial: Option<&EventTarget>) -> Result<Event, JsValue> {
        let event = self.document.create_event("CustomEvent")?;
        event.init_event_with_bubbles_and_cancelable(kind, false, true);
        let value = initial.map_or(JsValue::UNDEFINED, |target| JsValue::from(target.clone()));
        set(&event, "initialTarget", &value)?;
        number(&event, "tapCount", 0.0)?;
        Ok(event)
    }

    fn event_for_touch(&self, kind: &str, initial: Option<&EventTarget>) -> Result<Event, JsValue> {
        let event = self.event(kind, initial)?;
        if initial.is_none() {
            set(&event, "initialTarget", &JsValue::NULL)?;
        }
        Ok(event)
    }

    fn dispatch(&self, event: &Event, initial: Option<&EventTarget>) -> Result<(), JsValue> {
        match event.type_().as_str() {
            TAP => {
                let now = js_sys::Date::new_0().get_time();
                let count = self.tap_count.borrow_mut().tap(now);
                number(event, "tapCount", f64::from(count))?;
            }
            CHANGE | CONTEXT_MENU => self.tap_count.borrow_mut().reset(),
            _ => {}
        }
        let Some(initial) = initial.and_then(|target| target.dyn_ref::<Node>()) else {
            return Ok(());
        };
        if self
            .ignores
            .borrow()
            .iter()
            .any(|target| target.element.contains(Some(initial)))
        {
            return Ok(());
        }
        let mut targets: Vec<_> = self
            .targets
            .borrow()
            .iter()
            .filter_map(|target| {
                if !target.element.contains(Some(initial)) {
                    return None;
                }
                let mut node = Some(initial.clone());
                let mut depth = 0;
                while let Some(current) = node {
                    if current.is_same_node(Some(target.element.as_ref())) {
                        break;
                    }
                    depth += 1;
                    node = current.parent_element().map(Into::into);
                }
                Some((depth, target.element.clone()))
            })
            .collect();
        targets.sort_by_key(|entry| entry.0);
        for (_, target) in targets {
            target.dispatch_event(event)?;
            self.dispatched.set(true);
        }
        Ok(())
    }

    fn inertia_targets(&self, target: Option<&EventTarget>) -> Vec<EventTarget> {
        let Some(node) = target.and_then(|target| target.dyn_ref::<Node>()) else {
            return Vec::new();
        };
        self.targets
            .borrow()
            .iter()
            .filter(|target| target.element.contains(Some(node)))
            .map(|target| target.element.clone().into())
            .collect()
    }

    fn finish_batch(&self, event: &Event) {
        if self.dispatched.get() {
            event.prevent_default();
            event.stop_propagation();
            self.dispatched.set(false);
        }
    }

    fn inertia(
        self: &Rc<Self>,
        targets: Rc<Vec<EventTarget>>,
        state: Rc<RefCell<Inertia>>,
    ) -> Result<(), JsValue> {
        animation::schedule(self, targets, state)
    }
}

impl Drop for GestureContext {
    fn drop(&mut self) {
        // These are built-in removal/cancellation calls with valid owned handles.
        self.dispose()
            .expect("built-in gesture listener and frame cleanup cannot throw");
    }
}

fn set(target: &Event, name: &str, value: &JsValue) -> Result<(), JsValue> {
    if !js_sys::Reflect::set(target.as_ref(), &name.into(), value)? {
        return Err(error("Cannot set gesture event property"));
    }
    Ok(())
}
fn number(target: &Event, name: &str, value: f64) -> Result<(), JsValue> {
    set(target, name, &value.into())
}
fn error(value: &str) -> JsValue {
    js_sys::Error::new(value).into()
}
