// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Document-owned recognizer for browser/scrollable/touch.ts.
#[path = "touch_animation.rs"]
mod animation;
#[path = "touch_events.rs"]
mod events;
#[path = "touch_reads.rs"]
mod reads;
#[path = "touch_targets.rs"]
mod targets;
use targets::Targets;

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

type TouchRecord = Rc<RefCell<TouchData<JsValue>>>;
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
struct TouchId(u64);
/// Share one context per document through RuntimeContext. Listener closures keep
/// only Weak context references, and each registration token removes its own node.
pub struct GestureContext {
    document: Document,
    touch_device: bool,
    targets: RefCell<Targets>,
    ignores: RefCell<Targets>,
    active: RefCell<HashMap<TouchId, TouchRecord>>,
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
            targets: RefCell::new(Targets::default()),
            ignores: RefCell::new(Targets::default()),
            active: RefCell::new(HashMap::new()),
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
        Ok(self.register(element, false))
    }
    pub fn ignore_target(
        self: &Rc<Self>,
        element: &HtmlElement,
    ) -> Result<DisposableHandle, JsValue> {
        Ok(self.register(element, true))
    }
    pub fn is_touch_device(&self) -> bool {
        self.touch_device
    }

    fn register(self: &Rc<Self>, element: &HtmlElement, ignored: bool) -> DisposableHandle {
        if !self.touch_device {
            return NO_TOUCH.with(Rc::clone);
        }
        let entries = if ignored {
            &self.ignores
        } else {
            &self.targets
        };
        let target = Rc::downgrade(&entries.borrow_mut().push(element.clone()));
        let weak = Rc::downgrade(self);
        let removed = Cell::new(false);
        to_disposable(move || {
            if removed.replace(true) {
                return;
            }
            if let (Some(context), Some(target)) = (weak.upgrade(), target.upgrade()) {
                let entries = if ignored {
                    &context.ignores
                } else {
                    &context.targets
                };
                entries.borrow_mut().remove(&target);
            }
        })
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

    fn event(&self, kind: &str, initial: Option<&JsValue>) -> Result<Event, JsValue> {
        let event = self.document.create_event("CustomEvent")?;
        reads::function(event.as_ref(), "initEvent")?.call3(
            event.as_ref(),
            &kind.into(),
            &JsValue::FALSE,
            &JsValue::TRUE,
        )?;
        let value = initial.cloned().unwrap_or(JsValue::UNDEFINED);
        set(&event, "initialTarget", &value)?;
        number(&event, "tapCount", 0.0)?;
        Ok(event)
    }

    fn dispatch(&self, event: &Event) -> Result<(), JsValue> {
        if type_is(event, TAP)? {
            let now = reads::tap_time()?;
            let count = self.tap_count.borrow_mut().tap(now);
            number(event, "tapCount", f64::from(count))?;
        } else if type_is(event, CHANGE)? || type_is(event, CONTEXT_MENU)? {
            self.tap_count.borrow_mut().reset();
        }
        if !reads::field(event.as_ref(), "initialTarget")?.is_instance_of::<Node>() {
            return Ok(());
        }
        let ignores = self.ignores.borrow().iter();
        for target in ignores {
            if contains_event(&target.element, event)? {
                return Ok(());
            }
        }
        let candidates = self.targets.borrow().iter();
        let mut targets = Vec::new();
        for target in candidates {
            if !contains_event(&target.element, event)? {
                continue;
            }
            let mut node = reads::field(event.as_ref(), "initialTarget")?;
            let target_value = JsValue::from(target.element.clone());
            let mut depth = 0;
            while node.is_truthy() && node != target_value {
                depth += 1;
                node = reads::field(&node, "parentElement")?;
            }
            targets.push((depth, target.element.clone()));
        }
        targets.sort_by_key(|entry| entry.0);
        for (_, target) in targets {
            target.dispatch_event(event)?;
            self.dispatched.set(true);
        }
        Ok(())
    }

    fn inertia_targets(&self, initial: &JsValue) -> Result<Vec<EventTarget>, JsValue> {
        let targets: Vec<_> = self
            .targets
            .borrow()
            .iter()
            .map(|target| target.element.clone())
            .collect();
        let mut matching = Vec::new();
        for target in targets {
            if initial.is_instance_of::<Node>() && contains(&target, initial)? {
                matching.push(target.into());
            }
        }
        Ok(matching)
    }

    fn finish_batch(&self, event: &Event) -> Result<(), JsValue> {
        if self.dispatched.get() {
            reads::call0(event.as_ref(), "preventDefault")?;
            reads::call0(event.as_ref(), "stopPropagation")?;
            self.dispatched.set(false);
        }
        Ok(())
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

fn type_is(event: &Event, kind: &str) -> Result<bool, JsValue> {
    Ok(reads::field(event.as_ref(), "type")? == JsValue::from_str(kind))
}

fn contains_event(target: &HtmlElement, event: &Event) -> Result<bool, JsValue> {
    // JS evaluates the method before its argument, and checks callability only
    // after evaluating that argument. Both getters may change routing or throw.
    let callback = reads::field(target.as_ref(), "contains")?;
    let initial = reads::field(event.as_ref(), "initialTarget")?;
    Ok(reads::call1(&callback, target.as_ref(), &initial)?.is_truthy())
}

fn contains(target: &HtmlElement, node: &JsValue) -> Result<bool, JsValue> {
    let callback = reads::field(target.as_ref(), "contains")?;
    Ok(reads::call1(&callback, target.as_ref(), node)?.is_truthy())
}

fn set(target: &Event, name: &str, value: &JsValue) -> Result<(), JsValue> {
    if !js_sys::Reflect::set(target.as_ref(), &name.into(), value)? {
        return Err(js_sys::TypeError::new("Cannot assign gesture event property").into());
    }
    Ok(())
}
fn number(target: &Event, name: &str, value: f64) -> Result<(), JsValue> {
    set(target, name, &value.into())
}
fn error(value: &str) -> JsValue {
    js_sys::Error::new(value).into()
}
