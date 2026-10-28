use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::log_service::{
    BoundLogCall, CustomLogger, LogError, LogLevel, LogParameter, LogRecord, LogService, LogSink,
    LogValue,
};
use xterm_core::options::{OptionValue, OptionsService};

struct Dynamic {
    events: Rc<RefCell<Vec<&'static str>>>,
    version: Rc<Cell<usize>>,
    error: Option<Rc<dyn std::error::Error>>,
}
impl LogSink for Dynamic {
    fn log(&self, _: LogRecord) -> Result<(), LogError> {
        panic!("must use captured dynamic method")
    }
    fn prepare(&self, _: LogLevel) -> Result<Option<BoundLogCall>, LogError> {
        self.events.borrow_mut().push("bind");
        if let Some(error) = &self.error {
            return Err(LogError::Host(Rc::clone(error)));
        }
        let version = self.version.get();
        let events = Rc::clone(&self.events);
        Ok(Some(Box::new(move |record| {
            assert_eq!(version, 0, "method captured before lazy mutation");
            assert_eq!(
                record.message,
                "xterm.js: message".encode_utf16().collect::<Vec<_>>()
            );
            events.borrow_mut().push("call");
            Ok(())
        })))
    }
}

#[test]
fn dynamic_method_is_captured_before_lazy_callbacks_but_prefix_is_read_afterward() {
    let options = Rc::new(OptionsService::new(false, [("logLevel", "trace".into())]));
    let events = Rc::new(RefCell::new(vec![]));
    let version = Rc::new(Cell::new(0));
    let sink: Rc<dyn LogSink> = Rc::new(Dynamic {
        events: Rc::clone(&events),
        version: Rc::clone(&version),
        error: None,
    });
    options
        .set(
            "logger",
            OptionValue::object(CustomLogger(Rc::clone(&sink))),
        )
        .unwrap();
    let log = LogService::new(Rc::clone(&options), sink).unwrap();
    log.log(
        LogLevel::Info,
        &"message".encode_utf16().collect::<Vec<_>>(),
        vec![LogParameter::Lazy(Box::new(|| {
            events.borrow_mut().push("lazy");
            version.set(1);
            options.set("logger", OptionValue::Null).unwrap();
            LogValue::CodeUnits(vec![])
        }))],
    )
    .unwrap();
    assert_eq!(*events.borrow(), ["bind", "lazy", "call"]);
}

#[test]
fn method_getter_failure_prevents_lazy_arguments_and_retains_original_cause() {
    let options = Rc::new(OptionsService::new(false, [("logLevel", "trace".into())]));
    let events = Rc::new(RefCell::new(vec![]));
    let original: Rc<dyn std::error::Error> = Rc::new(std::io::Error::other("method getter threw"));
    let sink: Rc<dyn LogSink> = Rc::new(Dynamic {
        events: Rc::clone(&events),
        version: Rc::new(Cell::new(0)),
        error: Some(Rc::clone(&original)),
    });
    let log = LogService::new(options, sink).unwrap();
    let error = log
        .log(
            LogLevel::Info,
            &[],
            vec![LogParameter::Lazy(Box::new(|| {
                panic!("getter failed first")
            }))],
        )
        .unwrap_err();
    let LogError::Host(actual) = error else {
        panic!("cause category lost");
    };
    assert!(Rc::ptr_eq(&actual, &original));
    assert_eq!(*events.borrow(), ["bind"]);
}

#[test]
fn filtered_messages_do_not_access_dynamic_methods() {
    let options = Rc::new(OptionsService::new(false, [("logLevel", "off".into())]));
    let events = Rc::new(RefCell::new(vec![]));
    let sink: Rc<dyn LogSink> = Rc::new(Dynamic {
        events: Rc::clone(&events),
        version: Rc::new(Cell::new(0)),
        error: None,
    });
    let log = LogService::new(options, sink).unwrap();
    log.log(
        LogLevel::Error,
        &[],
        vec![LogParameter::Lazy(Box::new(|| panic!("filtered")))],
    )
    .unwrap();
    assert!(events.borrow().is_empty());
}

#[test]
fn property_key_views_accept_symbols_and_preserve_errors_without_string_conversion() {
    use xterm_core::{
        log_service::LogPropertyKey,
        options::{CoercionError, OptionError},
    };
    let options = Rc::new(OptionsService::default());
    let sink: Rc<dyn LogSink> = Rc::new(Dynamic {
        events: Rc::new(RefCell::new(vec![])),
        version: Rc::new(Cell::new(0)),
        error: None,
    });
    let log = LogService::new(Rc::clone(&options), sink).unwrap();
    let calls = Rc::new(Cell::new(0));
    let capture = Rc::clone(&calls);
    options
        .set(
            "logLevel",
            OptionValue::object(LogPropertyKey(Rc::new(move || {
                capture.set(capture.get() + 1);
                Ok(None)
            }))),
        )
        .unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(log.level(), None);
    options
        .set(
            "logLevel",
            OptionValue::object(LogPropertyKey(Rc::new(|| {
                Ok(Some("debug".encode_utf16().collect()))
            }))),
        )
        .unwrap();
    assert_eq!(log.level(), Some(LogLevel::Debug));
    let error = options
        .set(
            "logLevel",
            OptionValue::object(LogPropertyKey(Rc::new(|| {
                Err(OptionError::Coercion(CoercionError(vec![0xd800])))
            }))),
        )
        .unwrap_err();
    assert_eq!(error.message_utf16(), [0xd800]);
    assert_eq!(log.level(), Some(LogLevel::Debug));
}

type DropCallback = Box<dyn Fn()>;
struct DropProbe(RefCell<Option<DropCallback>>);
impl LogSink for DropProbe {
    fn log(&self, _: LogRecord) -> Result<(), LogError> {
        Err(LogError::Sink(
            "replaced console should not be invoked".into(),
        ))
    }
}
impl Drop for DropProbe {
    fn drop(&mut self) {
        if let Some(callback) = self.0.get_mut().take() {
            callback();
        }
    }
}
struct CountingSink(Rc<Cell<usize>>);
impl LogSink for CountingSink {
    fn log(&self, _: LogRecord) -> Result<(), LogError> {
        self.0.set(self.0.get() + 1);
        Ok(())
    }
}

#[test]
fn replacing_console_releases_borrow_before_old_sink_drop_reenters() {
    let options = Rc::new(OptionsService::default());
    let old = Rc::new(DropProbe(RefCell::new(None)));
    let logger = Rc::new(LogService::new(options, old.clone()).unwrap());
    let weak = Rc::downgrade(&logger);
    let first_calls = Rc::new(Cell::new(0));
    let second_calls = Rc::new(Cell::new(0));
    let second = Rc::clone(&second_calls);
    *old.0.borrow_mut() = Some(Box::new(move || {
        let logger = weak.upgrade().unwrap();
        logger.log(LogLevel::Info, &[], vec![]).unwrap();
        logger.set_console_sink(Rc::new(CountingSink(Rc::clone(&second))));
    }));
    drop(old);
    logger.set_console_sink(Rc::new(CountingSink(Rc::clone(&first_calls))));
    assert_eq!(first_calls.get(), 1);
    logger.log(LogLevel::Info, &[], vec![]).unwrap();
    assert_eq!(second_calls.get(), 1);
}
