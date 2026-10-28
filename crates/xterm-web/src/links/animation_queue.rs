// Copyright (c) 2026 The xterm.js authors. All rights reserved. MIT license.
use js_sys::{Array, Function, Reflect};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::Window;
use xterm_core::lifecycle::Disposable;

type FrameCallback = Closure<dyn FnMut(f64)>;
type Runner = Rc<dyn Fn(&WindowAnimationQueue) -> Result<(), JsValue>>;
struct Item {
    canceled: Cell<bool>,
    priority: Cell<f64>,
    runner: Runner,
}

/// Cancellation does not cancel the window's shared animation frame. Priority
/// remains mutable until execution, matching the upstream queue item contract.
#[derive(Clone)]
pub struct AnimationItem {
    item: Rc<Item>,
}
impl AnimationItem {
    pub fn set_priority(&self, priority: f64) {
        self.item.priority.set(priority);
    }
}
impl Disposable for AnimationItem {
    fn dispose(&self) {
        self.item.canceled.set(true);
    }
}

struct State {
    window: Window,
    next: RefCell<Vec<Rc<Item>>>,
    current: RefCell<Vec<Rc<Item>>>,
    requested: Cell<Option<i32>>,
    callback: RefCell<Option<FrameCallback>>,
}
impl State {
    fn run(self: &Rc<Self>) -> Result<(), JsValue> {
        self.requested.set(None);
        *self.current.borrow_mut() = std::mem::take(&mut *self.next.borrow_mut());
        loop {
            let item = {
                let mut current = self.current.borrow_mut();
                if current.is_empty() {
                    break;
                }
                // Use the host's ECMAScript stable sort so non-total comparators
                // involving NaN have the same browser-specific ordering as JS.
                // Only numeric indices cross this boundary; runners stay in Rust.
                let order = Array::new();
                for index in 0..current.len() {
                    order.push(&JsValue::from_f64(index as f64));
                }
                let priorities: Vec<_> = current.iter().map(|item| item.priority.get()).collect();
                let comparator = Closure::wrap(Box::new(move |a: JsValue, b: JsValue| {
                    let a = a.as_f64().map(|v| v as usize);
                    let b = b.as_f64().map(|v| v as usize);
                    match (
                        a.and_then(|i| priorities.get(i)),
                        b.and_then(|i| priorities.get(i)),
                    ) {
                        (Some(a), Some(b)) => b - a,
                        _ => f64::NAN,
                    }
                })
                    as Box<dyn FnMut(JsValue, JsValue) -> f64>);
                let sort: Function =
                    Reflect::get(order.as_ref(), &JsValue::from_str("sort"))?.dyn_into()?;
                sort.call1(order.as_ref(), comparator.as_ref())?;
                let sorted: Vec<_> = order
                    .iter()
                    .map(|index| {
                        index
                            .as_f64()
                            .filter(|value| value.fract() == 0.0 && *value >= 0.0)
                            .and_then(|index| current.get(index as usize))
                            .cloned()
                            .ok_or_else(|| JsValue::from_str("invalid animation sort index"))
                    })
                    .collect::<Result<_, _>>()?;
                *current = sorted;
                current.remove(0)
            };
            if !item.canceled.get() {
                if let Err(error) = (item.runner)(&WindowAnimationQueue {
                    state: Rc::clone(self),
                }) {
                    web_sys::console::error_1(&error);
                }
            }
        }
        Ok(())
    }
}
impl Drop for State {
    fn drop(&mut self) {
        if let Some(handle) = self.requested.take() {
            if let Err(error) = self.window.cancel_animation_frame(handle) {
                web_sys::console::error_1(&error);
            }
        }
        self.callback.get_mut().take();
    }
}

#[derive(Clone)]
pub struct WindowAnimationQueue {
    state: Rc<State>,
}
impl WindowAnimationQueue {
    fn new(window: Window) -> Self {
        let state = Rc::new(State {
            window,
            next: RefCell::new(Vec::new()),
            current: RefCell::new(Vec::new()),
            requested: Cell::new(None),
            callback: RefCell::new(None),
        });
        let weak = Rc::downgrade(&state);
        let callback = Closure::wrap(Box::new(move |_: f64| {
            if let Some(state) = weak.upgrade() {
                if let Err(error) = state.run() {
                    web_sys::console::error_1(&error);
                }
            }
        }) as Box<dyn FnMut(f64)>);
        *state.callback.borrow_mut() = Some(callback);
        Self { state }
    }
    pub fn schedule(
        &self,
        priority: f64,
        runner: impl Fn(&Self) -> Result<(), JsValue> + 'static,
    ) -> Result<AnimationItem, JsValue> {
        let item = Rc::new(Item {
            canceled: Cell::new(false),
            priority: Cell::new(priority),
            runner: Rc::new(runner),
        });
        self.state.next.borrow_mut().push(item.clone());
        if self.state.requested.get().is_none() {
            let callback = self.state.callback.borrow();
            let callback = callback
                .as_ref()
                .ok_or_else(|| JsValue::from_str("animation queue callback unavailable"))?;
            let handle = self
                .state
                .window
                .request_animation_frame(callback.as_ref().unchecked_ref())?;
            self.state.requested.set(Some(handle));
        }
        Ok(AnimationItem { item })
    }
}

/// The browser platform owns one registry and shares it with all terminals and
/// services. This replaces the source module's global Map with explicit ownership,
/// while still coalescing all callers targeting the same Window into one queue.
#[derive(Default)]
pub struct WindowAnimationQueues {
    queues: Vec<WindowAnimationQueue>,
}
impl WindowAnimationQueues {
    pub fn for_window(&mut self, window: &Window) -> WindowAnimationQueue {
        if let Some(queue) = self
            .queues
            .iter()
            .find(|queue| queue.state.window == *window)
        {
            return queue.clone();
        }
        let queue = WindowAnimationQueue::new(window.clone());
        self.queues.push(queue.clone());
        queue
    }
}
