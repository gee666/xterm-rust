//! Raw browser data keeps its JS identity. The portable core owns admission,
//! scrolling and user-input effects; this adapter owns host logging operations.
use crate::{
    js_error::{callback_error, to_js_error},
    utf16::from_js_string,
};
use js_sys::{Array, Function, JsString, Reflect};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use xterm_core::{
    core_service::CoreService,
    log_service::{LogError, LogLevel, LogParameter, LogValue},
};

#[wasm_bindgen(inline_js = "
export function inputProperty(value, key) { return value[key]; }
")]
extern "C" {
    #[wasm_bindgen(catch, js_name = inputProperty)]
    fn property(value: &JsValue, key: &str) -> Result<JsValue, JsValue>;
}

fn invoke(value: &JsValue, key: &str, arguments: &Array) -> Result<JsValue, JsValue> {
    let method = property(value, key)?;
    // Reflect.apply supplies the original receiver and ignores an own 'call'
    // property. checked conversion would reject cross-realm callable values.
    Reflect::apply(method.unchecked_ref::<Function>(), value, arguments)
}

fn codes(data: &JsValue) -> Result<LogValue, LogError> {
    let result = (|| {
        let split = invoke(data, "split", &Array::of1(&"".into()))?;
        let map = property(&split, "map")?;
        // A custom map may retain this callback. Transfer ownership to JS GC,
        // rather than invalidating the closure at the end of this log call.
        let character = Closure::<dyn Fn(JsValue) -> Result<JsValue, JsValue>>::new(|value| {
            invoke(&value, "charCodeAt", &Array::of1(&0.into()))
        })
        .into_js_value();
        let value = Reflect::apply(map.unchecked_ref(), &split, &Array::of1(&character))?;
        xterm_wasm_support::options::from_js(&value)
    })();
    result
        .map(LogValue::Value)
        .map_err(|error| LogError::Host(std::rc::Rc::new(crate::js_error::JsCallbackError(error))))
}

pub(super) fn send(
    core: &CoreService,
    data: &JsValue,
    user_input: bool,
    deliver: impl FnOnce(&JsValue) -> Result<(), JsValue>,
) -> Result<(), JsValue> {
    core.trigger_data_event_with(user_input, |logger| {
        // Template conversion is eager even when Debug is disabled. It may
        // throw or reenter before the logger reads its current level.
        let text = xterm_wasm_support::options::from_js(data)
            .map_err(callback_error)?
            .to_utf16()?;
        let mut message: Vec<u16> = "sending data \"".encode_utf16().collect();
        message.extend(text);
        message.push(u16::from(b'"'));
        logger.log(LogLevel::Debug, &message, Vec::new())?;
        logger.log(
            LogLevel::Trace,
            &"sending data (codes)".encode_utf16().collect::<Vec<_>>(),
            vec![LogParameter::TryLazy(Box::new(|| codes(data)))],
        )?;
        deliver(data).map_err(callback_error)?;
        Ok(())
    })
    .map_err(|error| to_js_error(&error))
}

/// The same browser emitter handles native strings and raw input. It is bound
/// before public subscriptions, and its lifetime follows the native data API.
pub(super) fn bind(
    native: &xterm_core::headless::Terminal,
    data: &std::rc::Rc<xterm_core::event::Emitter<JsValue>>,
) -> [xterm_core::lifecycle::DisposableHandle; 2] {
    let output = std::rc::Rc::clone(data);
    let forward = native
        .on_data()
        .try_subscribe(move |value| output.fire(&crate::utf16::to_js_string(value).into()));
    let output = std::rc::Rc::clone(data);
    let disposal = native.on_dispose_request().subscribe(move |_| {
        xterm_core::lifecycle::Disposable::dispose(output.as_ref());
    });
    [forward, disposal]
}

/// Normal string input still uses the native UTF-16 event path.
pub(super) fn text(data: &JsValue) -> Option<Vec<u16>> {
    data.is_string()
        .then(|| from_js_string(&JsString::from(data.clone())))
}
