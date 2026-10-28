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
