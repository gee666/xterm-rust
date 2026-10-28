// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Adapted from common/services/LogService.ts.

//! Portable logging policy. The host supplies console methods through `LogSink`;
//! no platform output API is coupled to the terminal core.

use crate::lifecycle::{Disposable, DisposableHandle};
use crate::options::{OptionError, OptionValue, OptionsService};
use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Off,
}

impl LogLevel {
    fn from_option(value: &OptionValue) -> Result<Option<Self>, OptionError> {
        // JS object indexing performs ToPropertyKey, including adapter-owned
        // objects whose primitive conversion returns a recognized level name.
        Ok(
            match String::from_utf16_lossy(&value.text_utf16()?).as_str() {
                "trace" => Some(Self::Trace),
                "debug" => Some(Self::Debug),
                "info" => Some(Self::Info),
                "warn" => Some(Self::Warn),
                "error" => Some(Self::Error),
                "off" => Some(Self::Off),
                _ => None,
            },
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogRoute {
    Log,
    Info,
    Warn,
    Error,
    Custom,
}

#[derive(Clone, Debug, PartialEq)]
pub enum LogValue {
    Value(OptionValue),
    CodeUnits(Vec<u16>),
}

pub enum LogParameter<'a> {
    Value(LogValue),
    Lazy(Box<dyn FnOnce() -> LogValue + 'a>),
    TryLazy(Box<dyn FnOnce() -> Result<LogValue, LogError> + 'a>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct LogRecord {
    pub level: LogLevel,
    pub route: LogRoute,
    pub message: Vec<u16>,
    pub parameters: Vec<LogValue>,
}

#[derive(Debug)]
pub enum LogError {
    Option(OptionError),
    InvalidLogger,
    Sink(String),
    Host(Rc<dyn std::error::Error>),
}
impl fmt::Display for LogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Option(error) => error.fmt(f),
            Self::InvalidLogger => f.write_str("logger does not provide callable logging methods"),
            Self::Sink(message) => f.write_str(message),
            Self::Host(error) => error.fmt(f),
        }
    }
}
impl std::error::Error for LogError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Option(error) => Some(error),
            Self::Host(error) => Some(error.as_ref()),
            Self::InvalidLogger | Self::Sink(_) => None,
        }
    }
}
impl From<OptionError> for LogError {
    fn from(error: OptionError) -> Self {
        Self::Option(error)
    }
}

/// A call may reenter the terminal. Implementations must not retain a mutable
/// borrow across user callbacks. Errors are the equivalent of JS logger throws.
pub trait LogSink {
    fn log(&self, record: LogRecord) -> Result<(), LogError>;
}

/// Store this payload in `OptionValue::object` to install a custom logger. The
/// adapter must provide all five methods, dispatching according to record.level.
#[derive(Clone)]
pub struct CustomLogger(pub Rc<dyn LogSink>);

pub struct LogService {
    options: Rc<OptionsService>,
    level: Rc<Cell<Option<LogLevel>>>,
    option_listener: DisposableHandle,
    console: Rc<dyn LogSink>,
}

impl LogService {
    pub fn new(options: Rc<OptionsService>, console: Rc<dyn LogSink>) -> Result<Self, OptionError> {
        let level = Rc::new(Cell::new(LogLevel::from_option(&options.get("logLevel")?)?));
        let weak = Rc::downgrade(&level);
        let option_listener = options.try_on_specific_option_change("logLevel", move |value| {
            if let Some(level) = weak.upgrade() {
                level.set(LogLevel::from_option(&value)?);
            }
            Ok(())
        })?;
        Ok(Self {
            options,
            level,
            option_listener,
            console,
        })
    }

    /// None models an invalid raw JS log level, whose comparisons suppress logs.
    pub fn level(&self) -> Option<LogLevel> {
        self.level.get()
    }

    pub fn log(
        &self,
        level: LogLevel,
        message: &[u16],
        parameters: Vec<LogParameter<'_>>,
    ) -> Result<(), LogError> {
        if level == LogLevel::Off || self.level().is_none_or(|minimum| minimum > level) {
            return Ok(());
        }
        // JS resolves the bound logging method before evaluating lazy arguments.
        let logger = self.options.get("logger")?;
        let (sink, route) = match &logger {
            OptionValue::Undefined | OptionValue::Null => (
                Rc::clone(&self.console),
                match level {
                    LogLevel::Trace | LogLevel::Debug => LogRoute::Log,
                    LogLevel::Info => LogRoute::Info,
                    LogLevel::Warn => LogRoute::Warn,
                    LogLevel::Error => LogRoute::Error,
                    LogLevel::Off => unreachable!(),
                },
            ),
            OptionValue::Object(object) => {
                let custom = object
                    .payload::<CustomLogger>()
                    .ok_or(LogError::InvalidLogger)?;
                (Rc::clone(&custom.0), LogRoute::Custom)
            }
            _ => return Err(LogError::InvalidLogger),
        };
        let parameters = parameters
            .into_iter()
            .map(|parameter| match parameter {
                LogParameter::Value(value) => Ok(value),
                LogParameter::Lazy(evaluate) => Ok(evaluate()),
                LogParameter::TryLazy(evaluate) => evaluate(),
            })
            .collect::<Result<Vec<_>, LogError>>()?;
        // A lazy argument can replace the logger; upstream reads prefix policy
        // after those callbacks, even though the output target is already bound.
        let mut output = if self.options.get("logger")?.is_truthy() {
            Vec::new()
        } else {
            "xterm.js: ".encode_utf16().collect()
        };
        output.extend_from_slice(message);
        sink.log(LogRecord {
            level,
            route,
            message: output,
            parameters,
        })
    }
}

impl Disposable for LogService {
    fn dispose(&self) {
        self.option_listener.dispose();
    }
}
impl Drop for LogService {
    fn drop(&mut self) {
        self.dispose();
    }
}
