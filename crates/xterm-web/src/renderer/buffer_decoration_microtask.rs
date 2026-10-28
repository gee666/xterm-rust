//! A one-shot host job is released after either enqueue failure or invocation.
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, prelude::*, JsValue};
use xterm_core::{
    decoration::{DecorationLineCache, DecorationService},
    lifecycle::DisposableHandle,
};
#[wasm_bindgen(module = "/src/renderer/decoration_microtask.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = queueDecorationSync)]
    fn queue_decoration_sync(run: &JsValue) -> Result<(), JsValue>;
}
/// Install at browser Terminal construction, not renderer open, so pre-open
/// marker changes and renderer replacement retain normal cache scheduling.
pub fn bind_microtasks(decorations: &Rc<DecorationService>) -> Result<DisposableHandle, JsValue> {
    let weak = Rc::downgrade(decorations);
    let cache = decorations.line_cache();
    let listener = cache.on_sync_requested().try_subscribe(move |_| {
        if let Some(decorations) = weak.upgrade() {
            schedule(decorations.line_cache()).map_err(crate::js_error::callback_error)?;
        }
        Ok(())
    });
    if cache.has_pending_sync() {
        if let Err(error) = schedule(cache) {
            listener.dispose();
            return Err(error);
        }
    }
    Ok(listener)
}
fn schedule(cache: DecorationLineCache) -> Result<(), JsValue> {
    // Native queueMicrotask runs exactly once. The bridge also invokes cancellation
    // on enqueue failure, so once_into_js never strands its captured Rust handle.
    let callback = Closure::once_into_js(move |cancelled: bool| {
        if cancelled {
            return JsValue::UNDEFINED;
        }
        match cache.flush_pending() {
            Ok(()) => JsValue::UNDEFINED,
            Err(error) => crate::js_error::to_js_error(&error),
        }
    });
    queue_decoration_sync(&callback)
}
