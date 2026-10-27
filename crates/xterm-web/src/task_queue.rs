// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
//! Live browser scheduling for the portable cooperative task queue.

use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use web_sys::{IdleDeadline, Performance, Window};
use xterm_core::task_queue::{CallbackKind, Scheduler, TaskQueue, WeakTaskQueue};

type Dispatch = Rc<RefCell<Option<WeakTaskQueue<BrowserScheduler>>>>;

enum CallbackId {
    Timer(i32),
    Idle(u32),
}

/// Keeps the WASM callback alive until it fires or is canceled. During dispatch,
/// wasm-bindgen holds an invocation reference while core drops this owner.
struct BrowserCallback {
    window: Window,
    id: CallbackId,
    _closure: Closure<dyn FnMut(JsValue)>,
}

impl Drop for BrowserCallback {
    fn drop(&mut self) {
        match self.id {
            CallbackId::Timer(id) => self.window.clear_timeout_with_handle(id),
            CallbackId::Idle(id) => self.window.cancel_idle_callback(id),
        }
    }
}

struct BrowserScheduler {
    window: Window,
    performance: Performance,
    dispatch: Dispatch,
}

impl Scheduler for BrowserScheduler {
    type Handle = BrowserCallback;

    fn request(&mut self, kind: CallbackKind) -> Self::Handle {
        let dispatch = Rc::clone(&self.dispatch);
        let callback = Closure::wrap(Box::new(move |deadline: JsValue| {
            // Release the dispatcher borrow before calling any task. Tasks may
            // enqueue/clear/flush or drop their final external queue owner.
            let queue = dispatch.borrow().as_ref().and_then(WeakTaskQueue::upgrade);
            if let Some(queue) = queue {
                match kind {
                    CallbackKind::Priority => queue.process_priority(),
                    CallbackKind::Idle => {
                        let deadline: IdleDeadline = deadline.unchecked_into();
                        queue.process(|| deadline.time_remaining());
                    }
                }
            }
        }) as Box<dyn FnMut(JsValue)>);
        let function = callback.as_ref().unchecked_ref();
        let id = match kind {
            CallbackKind::Priority => self
                .window
                .set_timeout_with_callback(function)
                .map(CallbackId::Timer),
            CallbackKind::Idle => self.window.request_idle_callback(function).map(CallbackId::Idle),
        };
        // Scheduler's portable trait has no error channel. Browser scheduling
        // failures are rethrown to JS, not disguised as a scheduled callback.
        let id = id.unwrap_or_else(|error| wasm_bindgen::throw_val(error));
        BrowserCallback {
            window: self.window.clone(),
            id,
            _closure: callback,
        }
    }

    fn cancel(&mut self, handle: Self::Handle) {
        drop(handle);
    }

    fn now(&self) -> f64 {
        self.performance.now()
    }

    fn deadline_exceeded(&mut self, milliseconds: f64) {
        web_sys::console::warn_1(
            &format!("task queue exceeded allotted deadline by {milliseconds}ms").into(),
        );
    }
}

/// A cloneable browser queue. Callbacks get a temporary queue handle for
/// reentrant operations; do not capture a strong queue in its own stored task.
#[derive(Clone)]
pub struct BrowserTaskQueue {
    queue: TaskQueue<BrowserScheduler>,
}

impl BrowserTaskQueue {
    pub fn priority(window: &Window) -> Result<Self, JsValue> {
        Self::new(window, CallbackKind::Priority)
    }

    pub fn idle(window: &Window) -> Result<Self, JsValue> {
        // Match upstream's property-existence test rather than guessing from
        // userAgent or treating a scheduling exception as an idle fallback.
        let supports_idle = js_sys::Reflect::has(window, &"requestIdleCallback".into())?;
        Self::new(
            window,
            if supports_idle { CallbackKind::Idle } else { CallbackKind::Priority },
        )
    }

    fn new(window: &Window, kind: CallbackKind) -> Result<Self, JsValue> {
        let performance = window.performance().ok_or_else(|| {
            js_sys::Error::new("Browser task queue requires performance.now")
        })?;
        let dispatch = Rc::new(RefCell::new(None));
        let queue = TaskQueue::new(
            BrowserScheduler {
                window: window.clone(),
                performance,
                dispatch: Rc::clone(&dispatch),
            },
            kind,
        );
        *dispatch.borrow_mut() = Some(queue.downgrade());
        Ok(Self { queue })
    }

    pub fn enqueue(&self, task: impl Fn(&Self) -> bool + 'static) {
        self.queue.enqueue(move |queue| task(&Self { queue: queue.clone() }));
    }

    pub fn flush(&self) {
        self.queue.flush();
    }

    pub fn clear(&self) {
        self.queue.clear();
    }

    pub fn has_pending_callback(&self) -> bool {
        self.queue.has_pending_callback()
    }

    pub fn remaining_tasks(&self) -> usize {
        self.queue.remaining_tasks()
    }
}
