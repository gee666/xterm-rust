// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Real browser callbacks drive the existing portable Buffer cleanup queue.
use crate::task_queue::BrowserTaskQueue;
use std::{
    any::Any,
    rc::{Rc, Weak},
};
use wasm_bindgen::JsValue;
use web_sys::Window;
use xterm_core::{
    buffer::{
        buffer_set::{BufferRef, CleanupFactory, CleanupFactoryHandle},
        BufferCleanup, BufferError,
    },
    event::EmitError,
    headless::Terminal,
    log_service::LogService,
};
use xterm_wasm_support::js_error::{to_js_error, JsCallbackError};

#[derive(Clone)]
enum Context {
    Source,
    Window(Window),
}

/// Attach before exposing the browser Terminal to user code. Source task queues
/// select their global runtime once, independently of the terminal's document.
pub fn install(terminal: &Rc<Terminal>) -> Result<CleanupFactoryHandle, JsValue> {
    install_context(terminal, Context::Source)
}

/// Explicit-window host injection, useful for iframe-owned embedders. The normal
/// xterm-compatible browser boundary uses install and its source-global policy.
pub fn install_in_window(
    terminal: &Rc<Terminal>,
    window: &Window,
) -> Result<CleanupFactoryHandle, JsValue> {
    install_context(terminal, Context::Window(window.clone()))
}
fn install_context(
    terminal: &Rc<Terminal>,
    context: Context,
) -> Result<CleanupFactoryHandle, JsValue> {
    let (buffers, logger) = terminal
        .with_state(|state| (Rc::clone(&state.buffer_service), Rc::clone(&state.logger)))
        .map_err(|error| to_js_error(&error))?;
    let logger = Rc::downgrade(&logger);
    let factory: CleanupFactory = Rc::new(move |buffer| attach(buffer, &logger, &context));
    buffers
        .install_cleanup_factory(factory)
        .map_err(|error| to_js_error(&error))
}
fn attach(
    buffer: &BufferRef,
    logger: &Weak<LogService>,
    context: &Context,
) -> Result<Box<dyn Any>, BufferError> {
    let logger = logger
        .upgrade()
        .ok_or(BufferError::InvalidOption("logger"))?;
    let queue = match context {
        Context::Source => BrowserTaskQueue::source_idle_with_logger(logger),
        Context::Window(window) => BrowserTaskQueue::idle_with_logger(window, logger),
    }
    .map_err(|error| BufferError::Dispatch(EmitError::callback(JsCallbackError(error))))?;
    // The browser scheduler already dispatches this exact queue from a weak
    // reference. The Buffer-owned adapter keeps it alive, without retaining its
    // Buffer or Terminal. Dispatch failures return through the real JS callback.
    let adapter = BufferCleanup::with_queue_map(
        buffer,
        queue.core_queue(),
        |error| JsCallbackError(to_js_error(&error)),
        |error| EmitError::Callback(error),
    )
    .map_err(|error| BufferError::Dispatch(EmitError::callback(error)))?;
    Ok(Box::new(adapter))
}
