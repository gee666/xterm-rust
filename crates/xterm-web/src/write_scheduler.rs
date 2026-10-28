//! Browser host for the portable write queue. A one-shot JavaScript callback
//! owns the queued work until invocation, matching source timer lifetimes.

use crate::js_error::{JsCallbackError, to_js_error};
use std::{future::Future, pin::Pin};
use wasm_bindgen::{closure::Closure, prelude::*};
use web_sys::{Performance, Window};
use xterm_core::input::write_buffer::{WeakWriteBuffer, WriteError, WriteScheduler};

#[wasm_bindgen(module = "/src/write_scheduler_bridge.js")]
extern "C" {
    #[wasm_bindgen(js_name = reportWriteError)]
    fn report_write_error(error: &JsValue);
    #[wasm_bindgen(catch, js_name = requestWrite)]
    fn request_write(callback: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = cancelWrite)]
    fn cancel_write(token: &JsValue) -> Result<(), JsValue>;
}

/// Source throws rejected parser exceptions in a JavaScript microtask, retaining
/// the actual exception value rather than producing an unhandled rejection.
pub fn report_unhandled_error(error: &JsValue) { report_write_error(error); }

/// Tokens have no Drop cancellation. Reentrant setTimeout can overwrite an older
/// token without canceling its callback in the source TimeoutTimer. Explicit
/// cancellation and queue destruction still call the scheduler's cancel method.
#[derive(Clone)]
pub struct WriteTimer { token: JsValue }

pub struct BrowserWriteScheduler { performance: Performance }
impl BrowserWriteScheduler {
    pub fn new(window: &Window) -> Result<Self, JsValue> {
        let performance = window.performance().ok_or_else(|| js_sys::Error::new("Writes require performance.now"))?;
        Ok(Self { performance })
    }
}
impl WriteScheduler for BrowserWriteScheduler {
    type Handle = WriteTimer;
    fn request(&self, queue: WeakWriteBuffer<Self>, last_time: f64, promise_result: bool) -> Result<Self::Handle, WriteError> {
        let queue = queue.upgrade().ok_or_else(|| WriteError::Schedule("Write queue expired before scheduling".into()))?;
        let callback = Closure::wrap(Box::new(move || {
            queue.process(last_time, promise_result).map_err(|error| to_js_error(&error))
        }) as Box<dyn Fn() -> Result<(), JsValue>>).into_js_value();
        // Transfer callback ownership before invoking the host: a setTimeout
        // shim may retain the function and then throw. Its function stays valid.
        let token = request_write(&callback).map_err(|error| WriteError::external(JsCallbackError(error)))?;
        Ok(WriteTimer { token })
    }
    fn cancel(&self, handle: Self::Handle) -> Result<(), WriteError> {
        cancel_write(&handle.token).map_err(|error| WriteError::external(JsCallbackError(error)))
    }
    fn now(&self) -> f64 { self.performance.now() }
    fn spawn(&self, future: Pin<Box<dyn Future<Output = ()>>>) { wasm_bindgen_futures::spawn_local(future); }
    fn report_error(&self, error: WriteError) { report_unhandled_error(&to_js_error(&error)); }
}
