//! Source-shaped callable subscriptions and disposal objects. No DOM ownership.
use crate::{js_error::callback_error, utf16::to_js_string};
use js_sys::Function;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use xterm_core::{
    event::{DispatchKey, Event},
    lifecycle::DisposableHandle,
};

#[wasm_bindgen(inline_js = "
export function noneDisposable() { return { dispose() {} }; }
export function collectDisposable(container, result) {
  if (container) {
    if (Array.isArray(container)) container.push(result);
    else container.add(result);
  }
}
export function sourceDisposable(callback) { return { dispose() { callback(); } }; }
export function callbackDisposable(callback) { return {dispose:callback}; }
")]
extern "C" {
    #[wasm_bindgen(js_name = noneDisposable)]
    fn none_disposable() -> JsValue;
    #[wasm_bindgen(js_name = sourceDisposable)]
    fn source_disposable(callback: &JsValue) -> JsValue;
    #[wasm_bindgen(js_name = callbackDisposable)]
    fn callback_disposable(callback: &JsValue) -> JsValue;
    #[wasm_bindgen(catch, js_name = collectDisposable)]
    fn collect_disposable(container: &JsValue, result: &JsValue) -> Result<(), JsValue>;
}

/// Preserve a supplied JS function's identity, return value, throw and receiver.
pub fn from_js_callback(callback: &Function) -> JsValue {
    callback_disposable(callback.as_ref())
}

/// Native callbacks capture their receiver explicitly. The closure is JS-owned
/// and supports recursive calls; unlike forget(), its lifetime can end at GC.
pub fn from_callback(callback: impl Fn() -> Result<JsValue, JsValue> + 'static) -> JsValue {
    let callback = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(callback).into_js_value();
    callback_disposable(&callback)
}

/// A source-shaped token. Its own dispose method can be extracted and invoked
/// without a receiver, unlike a this-bound exported WASM class method.
pub fn disposable(handle: DisposableHandle) -> JsValue {
    let callback = Closure::<dyn Fn()>::new(move || handle.dispose()).into_js_value();
    source_disposable(&callback)
}

/// JS owns the registration function closure via wasm-bindgen's finalizer.
/// Ignoring a returned subscription does not unregister its callback. Payload
/// conversion is shared by listeners in one dispatch, not retained between fires.
pub fn callable<T: 'static>(
    event: Event<T>,
    payload: impl Fn(&T) -> Result<JsValue, JsValue> + 'static,
) -> Function {
    let payload = std::rc::Rc::new(payload);
    let key = DispatchKey::<JsValue>::new();
    Closure::wrap(Box::new(
        move |listener: Function, this_args: JsValue, container: JsValue| {
            let payload = std::rc::Rc::clone(&payload);
            let key = key.clone();
            let registration = event.register_with_context(move |event, context| {
                let value =
                    context.try_memoize_with(&key, || payload(event).map_err(callback_error))?;
                listener
                    .call1(&this_args, &value)
                    .map(|_| ())
                    .map_err(callback_error)
            });
            if !registration.collect {
                return Ok(none_disposable());
            }
            let result = disposable(registration.handle);
            // Collection follows registration. A throwing push/add keeps the
            // listener installed, exactly as the original event function does.
            collect_disposable(&container, &result)?;
            Ok(result)
        },
    )
        as Box<dyn Fn(Function, JsValue, JsValue) -> Result<JsValue, JsValue>>)
    .into_js_value()
    .unchecked_into()
}

pub fn empty(_: &()) -> Result<JsValue, JsValue> {
    Ok(JsValue::UNDEFINED)
}
pub fn text(data: &[u16]) -> Result<JsValue, JsValue> {
    Ok(to_js_string(data).into())
}
