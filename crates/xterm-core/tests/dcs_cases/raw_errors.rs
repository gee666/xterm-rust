// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Raw streaming exceptions stop at the source action.
use crate::dcs_support::{params, Reports, IDENT};
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    event::EmitError,
    parser::{
        dcs::{DcsHandlerTrait, DcsParser},
        escape::{EscapeSequenceParser, FunctionIdentifier, ParseError},
        handler::{DispatchError, HandlerResult},
        params::Params,
        table::ParserState,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Hook,
    Put,
}
struct Stream {
    name: &'static str,
    failure: Option<Phase>,
    cause: EmitError,
    log: Reports,
}
impl Stream {
    fn record(&self, phase: Phase) -> Result<(), DispatchError> {
        self.log.borrow_mut().push(format!(
            "{}:{}",
            self.name,
            if phase == Phase::Hook { "H" } else { "P" }
        ));
        if self.failure == Some(phase) {
            Err(DispatchError::Callback(self.cause.clone()))
        } else {
            Ok(())
        }
    }
}
impl DcsHandlerTrait for Stream {
    fn hook(&mut self, _: &Params) -> Result<(), DispatchError> {
        self.record(Phase::Hook)
    }
    fn put(&mut self, _: &[u32]) -> Result<(), DispatchError> {
        self.record(Phase::Put)
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        self.log
            .borrow_mut()
            .push(format!("{}:U:{success}", self.name));
        HandlerResult::Ready(false)
    }
}
fn stream(
    name: &'static str,
    failure: Option<Phase>,
    cause: &EmitError,
    log: &Reports,
) -> Rc<RefCell<Stream>> {
    Rc::new(RefCell::new(Stream {
        name,
        failure,
        cause: cause.clone(),
        log: Rc::clone(log),
    }))
}
#[derive(Clone, Copy)]
enum EndMode {
    Plain,
    Pending,
    ThrowOnSuccess,
}
struct EndCase {
    name: &'static str,
    mode: EndMode,
    log: Reports,
    cause: EmitError,
}
impl DcsHandlerTrait for EndCase {
    fn hook(&mut self, _: &Params) -> Result<(), DispatchError> {
        Ok(())
    }
    fn put(&mut self, _: &[u32]) -> Result<(), DispatchError> {
        Ok(())
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        let message = format!("{}:U:{success}", self.name);
        if matches!(self.mode, EndMode::Pending) {
            let log = Rc::clone(&self.log);
            return HandlerResult::Pending(Box::pin(async move {
                log.borrow_mut().push(message);
                false
            }));
        }
        self.log.borrow_mut().push(message);
        if matches!(self.mode, EndMode::ThrowOnSuccess) && success {
            HandlerResult::Failed(DispatchError::Callback(self.cause.clone()))
        } else {
            HandlerResult::Ready(false)
        }
    }
}

#[test]
fn resumed_lower_failure_consumes_inner_pause_before_reset_like_source() {
    let cause = EmitError::callback(std::io::Error::other("resumed failure"));
    let log: Reports = Rc::default();
    let mut parser = DcsParser::new();
    for (name, mode) in [
        ("old", EndMode::Plain),
        ("middle", EndMode::ThrowOnSuccess),
        ("new", EndMode::Pending),
    ] {
        parser.register_handler(
            IDENT,
            Rc::new(RefCell::new(EndCase {
                name,
                mode,
                log: Rc::clone(&log),
                cause: cause.clone(),
            })),
        );
    }
    parser.hook(IDENT, &params(&[1])).unwrap();
    let pending = parser.unhook(true, true).unwrap().unwrap();
    assert!(!crate::dcs_support::resolve(pending));
    exact_cause(parser.unhook(true, false).err().unwrap(), &cause);
    parser.reset().unwrap();
    for cleanup in parser.take_cleanup() {
        assert!(!crate::dcs_support::resolve(cleanup));
    }
    assert_eq!(
        *log.borrow(),
        [
            "new:U:true",
            "middle:U:true",
            "middle:U:false",
            "old:U:false",
            "new:U:false"
        ]
    );
}

fn exact_cause(error: DispatchError, expected: &EmitError) {
    let DispatchError::Callback(actual) = error else {
        panic!("raw handler cause discarded")
    };
    assert_eq!(&actual, expected);
    assert!(std::ptr::eq(
        actual.callback_ref::<std::io::Error>().unwrap(),
        expected.callback_ref::<std::io::Error>().unwrap()
    ));
}

#[test]
fn raw_hook_and_put_failures_stop_older_handlers_without_delayed_unhook() {
    for phase in [Phase::Hook, Phase::Put] {
        let cause = EmitError::callback(std::io::Error::other("exact raw failure"));
        let log: Reports = Rc::default();
        let mut parser = DcsParser::new();
        parser.register_handler(IDENT, stream("old", None, &cause, &log));
        parser.register_handler(IDENT, stream("new", Some(phase), &cause, &log));
        let hook = parser.hook(IDENT, &params(&[1]));
        let error = if phase == Phase::Hook {
            hook.unwrap_err()
        } else {
            hook.unwrap();
            parser.put(&[65, 66]).unwrap_err()
        };
        exact_cause(error, &cause);
        let expected = if phase == Phase::Hook {
            vec!["new:H"]
        } else {
            vec!["new:H", "old:H", "new:P"]
        };
        assert_eq!(*log.borrow(), expected);
        parser.reset().unwrap();
        let mut expected = expected;
        expected.extend(["new:U:false", "old:U:false"]);
        assert_eq!(*log.borrow(), expected);
    }
}

#[test]
fn outer_parser_retains_source_state_and_stops_suffix_on_raw_streaming_errors() {
    for phase in [Phase::Hook, Phase::Put] {
        let cause = EmitError::callback(std::io::Error::other("exact raw failure"));
        let log: Reports = Rc::default();
        let mut parser = EscapeSequenceParser::new();
        parser.preceding_join_state.set(77);
        let out = Rc::clone(&log);
        parser.set_print_handler(move |_| out.borrow_mut().push("PRINT".into()));
        parser
            .register_dcs_handler(
                FunctionIdentifier::new("q"),
                stream("old", None, &cause, &log),
            )
            .unwrap();
        parser
            .register_dcs_handler(
                FunctionIdentifier::new("q"),
                stream("new", Some(phase), &cause, &log),
            )
            .unwrap();
        let data: Vec<u32> = "\x1bPqAB\u{9c}Z".chars().map(u32::from).collect();
        let error = parser.parse(&data, None).err().unwrap();
        let ParseError::Dispatch(error) = error else {
            panic!("raw error was not propagated")
        };
        exact_cause(error, &cause);
        assert!(!log
            .borrow()
            .iter()
            .any(|value| value == "PRINT" || value.contains(":U:")));
        assert_eq!(
            parser.current_state,
            if phase == Phase::Hook {
                ParserState::DcsEntry
            } else {
                ParserState::DcsPassthrough
            }
        );
        assert_eq!(parser.preceding_join_state.get(), 77);
        assert_eq!(parser.params().values(), [0]);
        assert_eq!(parser.collect(), 0);
        parser.reset().unwrap();
        parser
            .clear_dcs_handler(FunctionIdentifier::new("q"))
            .unwrap();
        assert!(parser.parse(&[90], None).unwrap().is_none());
        assert_eq!(log.borrow().last().map(String::as_str), Some("PRINT"));
    }
}
