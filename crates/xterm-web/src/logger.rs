//! JS logging methods are bound before lazy parameters run. This module only
//! adapts host values/calls; filtering, prefixes and ordering stay in Rust core.
use crate::{
    js_error::{callback_error, JsCallbackError},
    terminal::options,
    utf16::{from_js_string, to_js_string},
};
use js_sys::{Array, JsString};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use xterm_core::{
    log_service::{
        BoundLogCall, CustomLogger, LogError, LogLevel, LogPropertyKey, LogRecord, LogSink,
        LogValue,
    },
    options::{CoercionHint, OptionError, OptionObject, OptionValue},
};

#[wasm_bindgen(inline_js = "
export function xtermBindLogger(logger, method, fallback, options) { return logger[method].bind(options === undefined ? logger : options.logger) ?? console[fallback]; }
export function xtermConsoleMethod(method) { return console[method]; }
export function xtermInvokeLog(method, message, parameters) { method.call(console, message, ...parameters); }
export function xtermLoggerTruthy(value) { return !!value; }
export function xtermLogPropertyKey(value) { return Reflect.ownKeys({[value]:0})[0]; }
")]
extern "C" {
    #[wasm_bindgen(catch)]
    fn xtermBindLogger(
        logger: &JsValue,
        method: &str,
        fallback: &str,
        options: &JsValue,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn xtermConsoleMethod(method: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch)]
    fn xtermInvokeLog(
        method: &JsValue,
        message: &JsString,
        parameters: &Array,
    ) -> Result<(), JsValue>;
    fn xtermLoggerTruthy(value: &JsValue) -> bool;
    #[wasm_bindgen(catch)]
    fn xtermLogPropertyKey(value: &JsValue) -> Result<JsValue, JsValue>;
}

fn host_error(value: JsValue) -> LogError {
    LogError::Host(Rc::new(JsCallbackError(value)))
}
fn method(level: LogLevel, custom: bool) -> Result<&'static str, LogError> {
    Ok(match level {
        LogLevel::Trace if custom => "trace",
        LogLevel::Debug if custom => "debug",
        LogLevel::Trace | LogLevel::Debug => "log",
        LogLevel::Info => "info",
        LogLevel::Warn => "warn",
        LogLevel::Error => "error",
        LogLevel::Off => {
            return Err(LogError::Sink(
                "off is a filtering threshold, not a message level".into(),
            ))
        }
    })
}
fn invoke(bound: &JsValue, record: LogRecord) -> Result<(), LogError> {
    let parameters = Array::new();
    for parameter in record.parameters {
        let value = match parameter {
            LogValue::Value(value) => options::into_js(&value).map_err(host_error)?,
            LogValue::CodeUnits(units) => units
                .into_iter()
                .map(u32::from)
                .map(JsValue::from)
                .collect::<Array>()
                .into(),
        };
        parameters.push(&value);
    }
    xtermInvokeLog(bound, &to_js_string(&record.message), &parameters).map_err(host_error)
}

struct JsLogger {
    value: JsValue,
    options: Option<JsValue>,
}
impl JsLogger {
    fn bind(&self, level: LogLevel) -> Result<JsValue, LogError> {
        let options = self.options.clone().unwrap_or(JsValue::UNDEFINED);
        xtermBindLogger(
            &self.value,
            method(level, true)?,
            method(level, false)?,
            &options,
        )
        .map_err(host_error)
    }
}
impl LogSink for JsLogger {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        invoke(&self.bind(record.level)?, record)
    }
    fn prepare(&self, level: LogLevel) -> Result<Option<BoundLogCall>, LogError> {
        let bound = self.bind(level)?;
        Ok(Some(Box::new(move |record| invoke(&bound, record))))
    }
}

struct JsConsole;
impl LogSink for JsConsole {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        let bound = xtermConsoleMethod(method(record.level, false)?).map_err(host_error)?;
        invoke(&bound, record)
    }
    fn prepare(&self, level: LogLevel) -> Result<Option<BoundLogCall>, LogError> {
        let bound = xtermConsoleMethod(method(level, false)?).map_err(host_error)?;
        Ok(Some(Box::new(move |record| invoke(&bound, record))))
    }
}

/// Install this on the engine logger as its default console sink. Custom logger
/// options remain untouched, so source prefix and nullish fallback rules apply.
pub fn console_sink() -> Rc<dyn LogSink> {
    Rc::new(JsConsole)
}

/// Preserve raw JS identity/truthiness, including invalid primitive logger
/// values. Their native `.bind` TypeError occurs only when a message is admitted.
pub fn from_js(value: &JsValue) -> Result<OptionValue, JsValue> {
    convert(value, None)
}

/// The source reads the live logger again as the bind receiver after accessing
/// its method getter. That getter may replace the logger option synchronously.
pub fn from_js_with_options(
    value: &JsValue,
    raw_options: &js_sys::Object,
) -> Result<OptionValue, JsValue> {
    convert(value, Some(raw_options.clone().into()))
}

/// Log-level lookup uses ToPropertyKey rather than template-string conversion.
/// Symbol keys are valid and simply do not name one of the six source levels.
pub fn log_level_from_js(value: &JsValue) -> Result<OptionValue, JsValue> {
    let primitive = options::from_js(value)?;
    if !matches!(primitive, OptionValue::Object(_)) {
        return Ok(primitive);
    }
    let getter = primitive.clone();
    let key_value = value.clone();
    let key = LogPropertyKey(Rc::new(move || {
        let key = xtermLogPropertyKey(&key_value)
            .map_err(|error| OptionError::Dispatch(callback_error(error)))?;
        if key.is_symbol() {
            return Ok(None);
        }
        let key = key.dyn_into::<JsString>().map_err(|_| {
            OptionError::Dispatch(callback_error(
                js_sys::TypeError::new("ToPropertyKey returned neither a string nor a symbol")
                    .into(),
            ))
        })?;
        Ok(Some(from_js_string(&key)))
    }));
    Ok(OptionValue::Object(OptionObject::with_backing_value(
        key,
        value.clone(),
        move |hint| match hint {
            CoercionHint::Number => primitive.to_number().map(OptionValue::Number),
            CoercionHint::String => primitive.to_utf16().map(OptionValue::String),
        },
        move |_, key| getter.get_property(key),
        xtermLoggerTruthy(value),
        |a, b| a == b,
    )))
}

fn convert(value: &JsValue, raw_options: Option<JsValue>) -> Result<OptionValue, JsValue> {
    if value.is_null() || value.is_undefined() {
        return options::from_js(value);
    }
    let primitive = options::from_js(value)?;
    let getter = primitive.clone();
    let sink: Rc<dyn LogSink> = Rc::new(JsLogger {
        value: value.clone(),
        options: raw_options,
    });
    Ok(OptionValue::Object(OptionObject::with_backing_value(
        CustomLogger(sink),
        value.clone(),
        move |hint| match hint {
            CoercionHint::Number => primitive.to_number().map(OptionValue::Number),
            CoercionHint::String => primitive.to_utf16().map(OptionValue::String),
        },
        move |_, key| getter.get_property(key),
        xtermLoggerTruthy(value),
        |a, b| a == b,
    )))
}
