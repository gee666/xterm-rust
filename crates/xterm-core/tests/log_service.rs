use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::lifecycle::Disposable;
use xterm_core::log_service::{
    CustomLogger, LogError, LogLevel, LogParameter, LogRecord, LogRoute, LogService, LogSink,
    LogValue,
};
use xterm_core::options::{OptionObject, OptionValue, OptionsService};

#[derive(Default)]
struct Capture(RefCell<Vec<LogRecord>>);
impl LogSink for Capture {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        self.0.borrow_mut().push(record);
        Ok(())
    }
}
struct Callback(Box<dyn Fn(LogRecord) -> Result<(), LogError>>);
impl LogSink for Callback {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        (self.0)(record)
    }
}
fn message() -> Vec<u16> {
    vec![0xd800, 0, 0xd83d, 0xde00]
}
fn logger(options: &Rc<OptionsService>, sink: &Rc<Capture>) -> LogService {
    LogService::new(Rc::clone(options), sink.clone()).unwrap()
}
fn custom(options: &OptionsService, sink: Rc<dyn LogSink>) {
    options
        .set("logger", OptionValue::object(CustomLogger(sink)))
        .unwrap();
}

#[test]
fn every_level_threshold_and_console_route() {
    let options = Rc::new(OptionsService::default());
    let sink = Rc::new(Capture::default());
    let log = logger(&options, &sink);
    let levels = [
        LogLevel::Trace,
        LogLevel::Debug,
        LogLevel::Info,
        LogLevel::Warn,
        LogLevel::Error,
        LogLevel::Off,
    ];
    for (index, text) in ["trace", "debug", "info", "warn", "error", "off"]
        .iter()
        .enumerate()
    {
        options.set("logLevel", (*text).into()).unwrap();
        assert_eq!(log.level(), Some(levels[index]));
        for (message_level, level) in levels[..5].iter().copied().enumerate() {
            sink.0.borrow_mut().clear();
            let evaluated = Cell::new(0);
            log.log(
                level,
                &message(),
                vec![LogParameter::Lazy(Box::new(|| {
                    evaluated.set(evaluated.get() + 1);
                    LogValue::CodeUnits(vec![5])
                }))],
            )
            .unwrap();
            let allowed = index <= message_level;
            assert_eq!(evaluated.get(), usize::from(allowed));
            let records = sink.0.borrow();
            assert_eq!(records.len(), usize::from(allowed));
            if allowed {
                assert_eq!(
                    records[0].route,
                    [
                        LogRoute::Log,
                        LogRoute::Log,
                        LogRoute::Info,
                        LogRoute::Warn,
                        LogRoute::Error
                    ][message_level]
                );
                let mut expected: Vec<_> = "xterm.js: ".encode_utf16().collect();
                expected.extend(message());
                assert_eq!(records[0].message, expected);
                assert_eq!(records[0].parameters, [LogValue::CodeUnits(vec![5])]);
            }
        }
    }
}

#[test]
fn custom_logger_has_no_prefix_and_replacement_is_live() {
    let options = Rc::new(OptionsService::new(false, [("logLevel", "trace".into())]));
    let console = Rc::new(Capture::default());
    let first = Rc::new(Capture::default());
    let second = Rc::new(Capture::default());
    let log = logger(&options, &console);
    custom(&options, first.clone());
    log.log(LogLevel::Trace, &message(), vec![]).unwrap();
    custom(&options, second.clone());
    log.log(LogLevel::Warn, &[], vec![]).unwrap();
    assert_eq!(first.0.borrow()[0].route, LogRoute::Custom);
    assert_eq!(first.0.borrow()[0].message, message());
    assert_eq!(second.0.borrow()[0].route, LogRoute::Custom);
    assert!(second.0.borrow()[0].message.is_empty());
    assert!(console.0.borrow().is_empty());
}

#[test]
fn lazy_parameters_evaluate_once_left_to_right_and_can_replace_logger() {
    let options = Rc::new(OptionsService::new(false, [("logLevel", "trace".into())]));
    let console = Rc::new(Capture::default());
    let replacement = Rc::new(Capture::default());
    let log = logger(&options, &console);
    let order = RefCell::new(vec![]);
    log.log(
        LogLevel::Info,
        &message(),
        vec![
            LogParameter::Lazy(Box::new(|| {
                order.borrow_mut().push(1);
                custom(&options, replacement.clone());
                LogValue::CodeUnits(vec![1])
            })),
            LogParameter::Value(LogValue::Value(true.into())),
            LogParameter::Lazy(Box::new(|| {
                order.borrow_mut().push(2);
                LogValue::CodeUnits(vec![2])
            })),
        ],
    )
    .unwrap();
    assert_eq!(*order.borrow(), [1, 2]);
    assert_eq!(console.0.borrow()[0].message, message());
    assert_eq!(
        console.0.borrow()[0].parameters,
        [
            LogValue::CodeUnits(vec![1]),
            LogValue::Value(true.into()),
            LogValue::CodeUnits(vec![2])
        ]
    );
    assert!(replacement.0.borrow().is_empty());
    log.log(
        LogLevel::Info,
        &message(),
        vec![LogParameter::Lazy(Box::new(|| {
            options.set("logger", OptionValue::Null).unwrap();
            LogValue::CodeUnits(vec![])
        }))],
    )
    .unwrap();
    let mut expected: Vec<_> = "xterm.js: ".encode_utf16().collect();
    expected.extend(message());
    assert_eq!(replacement.0.borrow()[0].message, expected);
    assert_eq!(replacement.0.borrow()[0].route, LogRoute::Custom);
}

#[test]
fn invalid_levels_suppress_and_adapter_property_conversion_is_observed() {
    let options = Rc::new(OptionsService::default());
    let sink = Rc::new(Capture::default());
    let log = logger(&options, &sink);
    for value in [
        OptionValue::Undefined,
        OptionValue::Null,
        1.0.into(),
        "toString".into(),
        "TRACE".into(),
        "bad".into(),
    ] {
        options.set("logLevel", value).unwrap();
        assert_eq!(log.level(), None);
        log.log(
            LogLevel::Error,
            &[],
            vec![LogParameter::Lazy(Box::new(|| panic!("filtered")))],
        )
        .unwrap();
    }
    options
        .set(
            "logLevel",
            OptionValue::Object(OptionObject::new((), |_| Ok("debug".into()))),
        )
        .unwrap();
    assert_eq!(log.level(), Some(LogLevel::Debug));
    log.log(LogLevel::Debug, &[], vec![]).unwrap();
    assert_eq!(sink.0.borrow().len(), 1);
}

#[test]
fn raw_level_writes_do_not_emit_and_disposal_freezes_threshold() {
    let options = Rc::new(OptionsService::default());
    let sink = Rc::new(Capture::default());
    let log = logger(&options, &sink);
    options.set_raw("logLevel", "error".into()).unwrap();
    assert_eq!(log.level(), Some(LogLevel::Info));
    log.dispose();
    options.set("logLevel", "trace".into()).unwrap();
    assert_eq!(log.level(), Some(LogLevel::Info));
    log.log(LogLevel::Debug, &[], vec![]).unwrap();
    log.log(LogLevel::Info, &[], vec![]).unwrap();
    assert_eq!(sink.0.borrow().len(), 1);
}

#[test]
fn invalid_logger_and_sink_errors_propagate_before_later_work() {
    let options = Rc::new(OptionsService::new(false, [("logLevel", "trace".into())]));
    let sink = Rc::new(Capture::default());
    let log = logger(&options, &sink);
    for value in [
        false.into(),
        true.into(),
        1.0.into(),
        "".into(),
        OptionValue::object(()),
    ] {
        options.set("logger", value).unwrap();
        assert!(matches!(
            log.log(
                LogLevel::Trace,
                &[],
                vec![LogParameter::Lazy(Box::new(|| panic!("invalid logger")))]
            ),
            Err(LogError::InvalidLogger)
        ));
    }
    custom(
        &options,
        Rc::new(Callback(Box::new(|_| {
            Err(LogError::Sink("failure".into()))
        }))),
    );
    assert!(
        matches!(log.log(LogLevel::Info, &[], vec![]), Err(LogError::Sink(message)) if message == "failure")
    );
}

#[test]
fn sink_may_reenter_change_levels_and_dispose_service() {
    let options = Rc::new(OptionsService::new(false, [("logLevel", "trace".into())]));
    let console = Rc::new(Capture::default());
    let log = Rc::new(logger(&options, &console));
    let weak = Rc::downgrade(&log);
    let options_in_sink = Rc::clone(&options);
    custom(
        &options,
        Rc::new(Callback(Box::new(move |_| {
            let log = weak.upgrade().unwrap();
            options_in_sink.set("logger", OptionValue::Null).unwrap();
            options_in_sink.set("logLevel", "debug".into()).unwrap();
            log.log(LogLevel::Debug, &[], vec![])?;
            log.dispose();
            Ok(())
        }))),
    );
    log.log(LogLevel::Info, &[], vec![]).unwrap();
    options.set("logLevel", "off".into()).unwrap();
    assert_eq!(log.level(), Some(LogLevel::Debug));
    assert_eq!(console.0.borrow().len(), 1);
}

#[test]
fn coercion_errors_return_at_option_set_and_preserve_previous_level() {
    use xterm_core::options::{CoercionError, CoercionHint, OptionError};
    let options = Rc::new(OptionsService::default());
    let console = Rc::new(Capture::default());
    let log = logger(&options, &console);
    let failed = OptionValue::Object(OptionObject::new((), |hint| {
        assert_eq!(hint, CoercionHint::String);
        Err(CoercionError(vec![0xd800, 0]))
    }));
    let later_called = Rc::new(Cell::new(false));
    let capture = Rc::clone(&later_called);
    options
        .on_option_change()
        .subscribe(move |_| capture.set(true));
    let error = options.set("logLevel", failed.clone()).unwrap_err();
    assert_eq!(error.message_utf16(), [0xd800, 0]);
    assert_eq!(options.get("logLevel").unwrap(), failed);
    assert!(!later_called.get());
    assert_eq!(log.level(), Some(LogLevel::Info));
    log.log(LogLevel::Info, &[], vec![]).unwrap();
    assert_eq!(console.0.borrow().len(), 1);
    let invalid_options = Rc::new(OptionsService::new(false, [("logLevel", failed)]));
    assert!(matches!(
        LogService::new(invalid_options, console),
        Err(OptionError::Coercion(_))
    ));
}

#[test]
fn lazy_errors_stop_evaluation_and_do_not_reach_sink() {
    let options = Rc::new(OptionsService::new(false, [("logLevel", "trace".into())]));
    let sink = Rc::new(Capture::default());
    let log = logger(&options, &sink);
    let error = log
        .log(
            LogLevel::Info,
            &[],
            vec![
                LogParameter::TryLazy(Box::new(|| Err(LogError::Sink("lazy failure".into())))),
                LogParameter::Lazy(Box::new(|| panic!("must not evaluate after error"))),
            ],
        )
        .unwrap_err();
    assert!(matches!(error, LogError::Sink(message) if message == "lazy failure"));
    assert!(sink.0.borrow().is_empty());
}
