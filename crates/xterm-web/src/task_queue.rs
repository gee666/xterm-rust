// Copyright (c) 2022 The xterm.js authors. All rights reserved. MIT license.
//! Browser host for cooperative queues. JS owns scheduled callback lifetimes.
use crate::js_error::{to_js_error, JsCallbackError};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use web_sys::{IdleDeadline, Performance, Window};
use xterm_core::task_queue::{CallbackKind, Scheduler, TaskQueue, WeakTaskQueue};

type Dispatch = Rc<RefCell<Option<WeakTaskQueue<BrowserScheduler>>>>;

#[wasm_bindgen(module = "/src/task_queue_bridge.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = requireQueueRuntime)]
    fn require_queue_runtime() -> Result<(), JsValue>;
    #[wasm_bindgen(js_name = sourceIdleSupported)]
    fn source_idle_supported() -> bool;
    #[wasm_bindgen(catch, js_name = warnQueue)]
    fn warn_queue(message: &str) -> Result<(), JsValue>;
}

/// No Drop cancellation: a reentrant host request may overwrite the current
/// token without cancelling an earlier callback in the reference implementation.
#[derive(Clone, Copy)]
pub(crate) enum CallbackId {
    Timer(i32),
    Idle(u32),
}

pub(crate) struct BrowserScheduler {
    window: Window,
    performance: Performance,
    dispatch: Dispatch,
    logger: Option<Rc<xterm_core::log_service::LogService>>,
}
impl Scheduler for BrowserScheduler {
    type Handle = CallbackId;
    type Error = JsCallbackError;
    fn request(&self, kind: CallbackKind) -> Result<Self::Handle, Self::Error> {
        let dispatch = Rc::clone(&self.dispatch);
        let callback = Closure::wrap(Box::new(move |deadline: JsValue| {
            let queue = dispatch.borrow().as_ref().and_then(WeakTaskQueue::upgrade);
            let Some(queue) = queue else {
                return Ok(());
            };
            let result = match kind {
                CallbackKind::Priority => queue.process_priority(),
                CallbackKind::Idle => {
                    // requestIdleCallback supplies this WebIDL interface. A
                    // global instanceof check rejects valid iframe deadlines.
                    let deadline: IdleDeadline = deadline.unchecked_into();
                    queue.process(|| deadline.time_remaining())
                }
            };
            result.map_err(|error| error.0)
        }) as Box<dyn Fn(JsValue) -> Result<(), JsValue>>)
        .into_js_value();
        // Transfer ownership before the host call. A timer shim can retain the
        // function and then throw; the retained function must remain callable.
        // The checked ES2021 GC facilities reclaim cancelled, uninvoked closures.
        let function = callback.unchecked_ref();
        match kind {
            CallbackKind::Priority => self
                .window
                .set_timeout_with_callback(function)
                .map(CallbackId::Timer),
            CallbackKind::Idle => self
                .window
                .request_idle_callback(function)
                .map(CallbackId::Idle),
        }
        .map_err(JsCallbackError)
    }
    fn cancel(&self, handle: Self::Handle) {
        match handle {
            CallbackId::Timer(id) => self.window.clear_timeout_with_handle(id),
            CallbackId::Idle(id) => self.window.cancel_idle_callback(id),
        }
    }
    fn now(&self) -> f64 {
        self.performance.now()
    }
    fn deadline_exceeded(&self, milliseconds: f64) -> Result<(), Self::Error> {
        let message = format!(
            "task queue exceeded allotted deadline by {}ms",
            ryu_js::Buffer::new().format(milliseconds)
        );
        if let Some(logger) = &self.logger {
            logger
                .log(
                    xterm_core::log_service::LogLevel::Warn,
                    &message.encode_utf16().collect::<Vec<_>>(),
                    Vec::new(),
                )
                .map_err(|error| JsCallbackError(to_js_error(&error)))
        } else {
            warn_queue(&message).map_err(JsCallbackError)
        }
    }
}

/// Callbacks receive the queue for reentry; avoid capturing a strong queue in
/// one of its own stored tasks. Dropping its last owner cancels current work.
#[derive(Clone)]
pub struct BrowserTaskQueue {
    queue: TaskQueue<BrowserScheduler>,
}
impl BrowserTaskQueue {
    /// Explicit-window host injection. Source compatibility at the application
    /// boundary uses source_idle, which retains the module's global selection.
    pub fn priority(window: &Window) -> Result<Self, JsValue> {
        Self::new(window, CallbackKind::Priority, None)
    }
    pub fn idle(window: &Window) -> Result<Self, JsValue> {
        let supported = js_sys::Reflect::has(window, &"requestIdleCallback".into())?;
        Self::new(window, kind(supported), None)
    }
    pub fn priority_with_logger(
        window: &Window,
        logger: Rc<xterm_core::log_service::LogService>,
    ) -> Result<Self, JsValue> {
        Self::new(window, CallbackKind::Priority, Some(logger))
    }
    pub fn idle_with_logger(
        window: &Window,
        logger: Rc<xterm_core::log_service::LogService>,
    ) -> Result<Self, JsValue> {
        let supported = js_sys::Reflect::has(window, &"requestIdleCallback".into())?;
        Self::new(window, kind(supported), Some(logger))
    }
    /// Mirrors TaskQueue.ts's module-level capability choice and global timers,
    /// rather than resampling another document's capabilities on every queue.
    pub fn source_idle() -> Result<Self, JsValue> {
        Self::source(None)
    }
    pub fn source_idle_with_logger(
        logger: Rc<xterm_core::log_service::LogService>,
    ) -> Result<Self, JsValue> {
        Self::source(Some(logger))
    }
    fn source(logger: Option<Rc<xterm_core::log_service::LogService>>) -> Result<Self, JsValue> {
        let window = web_sys::window()
            .ok_or_else(|| js_sys::Error::new("Browser task queues require a Window"))?;
        Self::new(&window, kind(source_idle_supported()), logger)
    }
    fn new(
        window: &Window,
        kind: CallbackKind,
        logger: Option<Rc<xterm_core::log_service::LogService>>,
    ) -> Result<Self, JsValue> {
        require_queue_runtime()?;
        let performance = window
            .performance()
            .ok_or_else(|| js_sys::Error::new("Browser task queues require performance.now"))?;
        let dispatch = Rc::new(RefCell::new(None));
        let queue = TaskQueue::new(
            BrowserScheduler {
                window: window.clone(),
                performance,
                dispatch: dispatch.clone(),
                logger,
            },
            kind,
        );
        *dispatch.borrow_mut() = Some(queue.downgrade());
        Ok(Self { queue })
    }
    pub fn enqueue(&self, task: impl Fn(&Self) -> bool + 'static) -> Result<(), JsValue> {
        self.enqueue_try(move |queue| Ok(task(queue)))
    }
    pub fn enqueue_try(
        &self,
        task: impl Fn(&Self) -> Result<bool, JsValue> + 'static,
    ) -> Result<(), JsValue> {
        self.queue
            .enqueue_try(move |queue| {
                task(&Self {
                    queue: queue.clone(),
                })
                .map_err(JsCallbackError)
            })
            .map_err(|error| error.0)
    }
    pub fn set(&self, task: impl Fn(&Self) -> bool + 'static) -> Result<(), JsValue> {
        self.set_try(move |queue| Ok(task(queue)))
    }
    pub fn set_try(
        &self,
        task: impl Fn(&Self) -> Result<bool, JsValue> + 'static,
    ) -> Result<(), JsValue> {
        self.clear();
        self.enqueue_try(task)
    }
    pub fn flush(&self) -> Result<(), JsValue> {
        self.queue.flush().map_err(|error| error.0)
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
    pub(crate) fn core_queue(&self) -> TaskQueue<BrowserScheduler> {
        self.queue.clone()
    }
}
fn kind(supported: bool) -> CallbackKind {
    if supported {
        CallbackKind::Idle
    } else {
        CallbackKind::Priority
    }
}
