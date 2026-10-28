//! Diagnostic callback contracts. Live Params identity is covered by task 226,
//! not by the deliberately opaque payload used in these field/abort tests.
use std::{cell::Cell, rc::Rc};
use xterm_core::{
    log_service::{LogError, LogLevel, LogPropertyHost, LogRecord, LogService, LogSink, LogValue},
    options::{OptionValue, OptionsService},
    parser::{escape::ParsingState, params::Params, table::ParserState},
    terminal::diagnostics::log_error_state,
};
struct AbortSink;
impl LogSink for AbortSink {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        assert_eq!(record.level, LogLevel::Error);
        let LogValue::Object(fields) = &record.parameters[0] else {
            panic!("expected state object")
        };
        assert_eq!(
            fields
                .iter()
                .map(|(key, _)| key.as_str())
                .collect::<Vec<_>>(),
            [
                "position",
                "code",
                "currentState",
                "collect",
                "params",
                "abort"
            ]
        );
        let LogValue::Property(abort) = &fields[5].1 else {
            panic!("expected live abort property")
        };
        abort.set("abort".into())
    }
}
fn state(params: &Params) -> ParsingState<'_> {
    ParsingState {
        position: 2,
        code: 0x9f,
        current_state: ParserState::Ground,
        collect: 0,
        params,
    }
}
#[test]
fn logger_can_set_abort_with_source_truthiness() {
    let logger = LogService::new(Rc::new(OptionsService::default()), Rc::new(AbortSink)).unwrap();
    let params = Params::default();
    assert!(log_error_state(
        &logger,
        &state(&params),
        LogValue::Value(OptionValue::Undefined)
    )
    .unwrap());
}
struct FailedGetter {
    calls: Rc<Cell<usize>>,
    original: Rc<dyn std::error::Error>,
}
impl LogPropertyHost for FailedGetter {
    fn get(&self) -> Result<OptionValue, LogError> {
        self.calls.set(self.calls.get() + 1);
        Err(LogError::Host(Rc::clone(&self.original)))
    }
    fn set(&self, _: &OptionValue) -> Result<(), LogError> {
        Err(LogError::Host(Rc::clone(&self.original)))
    }
}
struct InstallGetter {
    calls: Rc<Cell<usize>>,
    original: Rc<dyn std::error::Error>,
}
impl LogSink for InstallGetter {
    fn log(&self, record: LogRecord) -> Result<(), LogError> {
        let LogValue::Object(fields) = &record.parameters[0] else {
            panic!("state object missing")
        };
        let LogValue::Property(abort) = &fields[5].1 else {
            panic!("abort property missing")
        };
        abort.install_host(Rc::new(FailedGetter {
            calls: Rc::clone(&self.calls),
            original: Rc::clone(&self.original),
        }));
        assert_eq!(
            self.calls.get(),
            0,
            "getter must not run inside logging callback"
        );
        Ok(())
    }
}
#[test]
fn abort_getter_runs_after_logger_and_preserves_original_error() {
    let calls = Rc::new(Cell::new(0));
    let original: Rc<dyn std::error::Error> =
        Rc::new(std::io::Error::other("abort accessor failed"));
    let sink = Rc::new(InstallGetter {
        calls: Rc::clone(&calls),
        original: Rc::clone(&original),
    });
    let logger = LogService::new(Rc::new(OptionsService::default()), sink).unwrap();
    let params = Params::default();
    let error = log_error_state(
        &logger,
        &state(&params),
        LogValue::Value(OptionValue::Undefined),
    )
    .unwrap_err();
    let LogError::Host(actual) = error else {
        panic!("error category lost")
    };
    assert!(Rc::ptr_eq(&actual, &original));
    assert_eq!(calls.get(), 1);
}
