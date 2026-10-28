// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Expected callbacks and state come from the unchanged JS outer parser.
use std::cell::RefCell;
use std::fmt::LowerHex;
use std::future::Future;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use xterm_core::lifecycle::DisposableHandle;
use xterm_core::parser::dcs::{DcsAction, DcsHandler, DcsHandlerTrait, HandlerHandle};
use xterm_core::parser::escape::{EscapeSequenceParser, FunctionIdentifier, ParseError};
use xterm_core::parser::handler::{DispatchError, HandlerResult};
use xterm_core::parser::params::{ParamValue, Params};

type Log = Rc<RefCell<Vec<String>>>;
fn hex<T: LowerHex>(data: &[T]) -> String {
    data.iter()
        .map(|value| format!("{value:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
fn params(value: &Params) -> String {
    value
        .to_array()
        .iter()
        .map(|value| match value {
            ParamValue::Param(value) => format!("p{value}"),
            ParamValue::SubParams(values) => format!(
                "s{}",
                values
                    .iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        })
        .collect::<Vec<_>>()
        .join(";")
}
struct Reporter {
    name: String,
    config: u8,
    index: usize,
    log: Log,
}
impl DcsHandlerTrait for Reporter {
    fn hook(&mut self, value: &Params) -> Result<(), DispatchError> {
        self.log
            .borrow_mut()
            .push(format!("H:{}:{}", self.name, params(value)));
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.log
            .borrow_mut()
            .push(format!("D:{}:{}", self.name, hex(data)));
        Ok(())
    }
    fn unhook(&mut self, success: bool) -> HandlerResult {
        let message = format!("U:{}:{}", self.name, u8::from(success));
        let handled = self.config != 1 || self.index == 0;
        if self.config != 0 && self.index == 1 {
            let log = Rc::clone(&self.log);
            HandlerResult::Pending(Box::pin(async move {
                log.borrow_mut().push(message);
                handled
            }))
        } else {
            self.log.borrow_mut().push(message);
            HandlerResult::Ready(handled)
        }
    }
}
fn parser(config: u8) -> (EscapeSequenceParser, Log) {
    let mut parser = EscapeSequenceParser::new();
    let log: Log = Rc::default();
    let output = Rc::clone(&log);
    parser.set_print_handler(move |data| output.borrow_mut().push(format!("P:{}", hex(data))));
    let output = Rc::clone(&log);
    parser.set_execute_handler_fallback(move |code| output.borrow_mut().push(format!("X:{code}")));
    let output = Rc::clone(&log);
    parser.set_esc_handler_fallback(move |id| output.borrow_mut().push(format!("E:{id}")));
    let output = Rc::clone(&log);
    parser.set_csi_handler_fallback(move |id, value| {
        output
            .borrow_mut()
            .push(format!("C:{id}:{}", params(value)))
    });
    let output = Rc::clone(&log);
    parser.dcs.set_handler_fallback(move |ident, action| {
        let value = match action {
            DcsAction::Hook(value) => format!("H:{}", params(&value)),
            DcsAction::Put(data) => format!("P:{}", hex(&data)),
            DcsAction::Unhook(success) => format!("U:{}", u8::from(success)),
        };
        output
            .borrow_mut()
            .push(format!("F:{}:{value}", ident as u32));
    });
    for (name, prefix, intermediates) in [
        ("q", None, None),
        ("+q", None, Some("+")),
        ("?+q", Some("?"), Some("+")),
    ] {
        let count = if config == 0 { 1 } else { 3 };
        for index in 0..count {
            let handler: HandlerHandle = Rc::new(RefCell::new(Reporter {
                name: format!("{name}{index}"),
                config,
                index,
                log: Rc::clone(&log),
            }));
            parser
                .register_dcs_handler(
                    FunctionIdentifier {
                        prefix,
                        intermediates,
                        final_byte: "q",
                    },
                    handler,
                )
                .unwrap();
        }
    }
    (parser, log)
}
struct NoWake;
impl Wake for NoWake {
    fn wake(self: Arc<Self>) {}
}
fn resolve(result: HandlerResult) -> Result<bool, DispatchError> {
    let mut future = Box::pin(result.resolve());
    let waker = Waker::from(Arc::new(NoWake));
    match future.as_mut().poll(&mut Context::from_waker(&waker)) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fixture future requires no external scheduling"),
    }
}
fn parse(parser: &mut EscapeSequenceParser, data: &[u32]) -> usize {
    let mut previous = None;
    let mut pauses = 0;
    while let Some(pending) = parser.parse(data, previous).unwrap() {
        // A nested DCS hook can queue old cleanup before returning this pending
        // callback. JS starts those promises eagerly in this creation order.
        for cleanup in parser.take_cleanup() {
            resolve(cleanup).unwrap();
        }
        previous = Some(resolve(pending).unwrap());
        pauses += 1;
    }
    // Match the oracle's executor turn even when parsing completed synchronously.
    for cleanup in parser.take_cleanup() {
        resolve(cleanup).unwrap();
    }
    pauses
}
#[test]
fn outer_parser_dcs_reference_split_and_termination_matrix() {
    let mut traces = 0;
    let mut calls = 0;
    let mut pauses = 0;
    for (case, line) in include_str!("fixtures/escape-dcs.tsv").lines().enumerate() {
        let mut fields = line.split('\t');
        let config = fields.next().unwrap().parse().unwrap();
        let operations = fields.next().unwrap();
        let expected = fields.next().unwrap();
        assert!(fields.next().is_none());
        let (mut parser, log) = parser(config);
        let mut snapshots = Vec::new();
        for operation in operations.split('|') {
            log.borrow_mut().clear();
            match operation {
                "r" => {
                    parser.reset().unwrap();
                    for pending in parser.take_cleanup() {
                        resolve(pending).unwrap();
                    }
                }
                "d" => parser.dispose(),
                _ => {
                    let data: Vec<u32> = operation[1..]
                        .split(',')
                        .filter(|value| !value.is_empty())
                        .map(|value| u32::from_str_radix(value, 16).unwrap())
                        .collect();
                    pauses += parse(&mut parser, &data);
                    calls += 1;
                }
            }
            snapshots.push(format!(
                "{}/{}/{}/{}/{}",
                parser.current_state as u8,
                parser.collect(),
                params(parser.params()),
                parser.preceding_join_state.get(),
                log.borrow().join("|")
            ));
        }
        assert_eq!(
            snapshots.join("~"),
            expected,
            "DCS integration trace {case}, config {config}"
        );
        traces += 1;
    }
    assert_eq!((traces, calls, pauses), (735, 1893, 454));
}
#[test]
fn reset_while_dcs_pending_cleans_remaining_handler_and_resumes_suffix_once() {
    let (mut parser, log) = parser(1);
    let data: Vec<u32> = "\x1bPqA\u{9c}Z".chars().map(u32::from).collect();
    let pending = parser.parse(&data, None).unwrap().unwrap();
    assert_eq!(resolve(pending), Ok(false));
    parser.reset().unwrap();
    assert!(parser.parse(&data, Some(false)).unwrap().is_none());
    assert_eq!(
        *log.borrow(),
        [
            "H:q2:p0", "H:q1:p0", "H:q0:p0", "D:q2:41", "D:q1:41", "D:q0:41", "U:q2:1", "U:q1:1",
            "U:q0:0", "P:5a",
        ]
    );
}

#[test]
fn rejected_dcs_future_stops_suffix_until_reset_and_saved_chunk_resumption() {
    let mut parser = EscapeSequenceParser::new();
    let printed: Rc<RefCell<Vec<u32>>> = Rc::default();
    let output = Rc::clone(&printed);
    parser.set_print_handler(move |data| output.borrow_mut().extend_from_slice(data));
    parser
        .register_dcs_handler(
            FunctionIdentifier::new("q"),
            Rc::new(RefCell::new(DcsHandler::new(|_, _| {
                HandlerResult::PendingFallible(Box::pin(async {
                    Err(DispatchError::CallbackFailed)
                }))
            }))),
        )
        .unwrap();
    let data: Vec<u32> = "\x1bPqpayload\u{9c}SUFFIX".chars().map(u32::from).collect();
    let pending = parser.parse(&data, None).unwrap().unwrap();
    assert_eq!(resolve(pending), Err(DispatchError::CallbackFailed));
    assert!(printed.borrow().is_empty());
    parser.reset().unwrap();
    parser
        .clear_dcs_handler(FunctionIdentifier::new("q"))
        .unwrap();
    assert!(parser.parse(&data, Some(false)).unwrap().is_none());
    assert_eq!(
        *printed.borrow(),
        "SUFFIX".chars().map(u32::from).collect::<Vec<_>>()
    );
    assert_eq!(parse(&mut parser, &[65]), 0);
    assert_eq!(
        *printed.borrow(),
        "SUFFIXA".chars().map(u32::from).collect::<Vec<_>>()
    );
}

#[test]
fn removed_active_handler_error_propagates_through_the_outer_parser() {
    let mut parser = EscapeSequenceParser::new();
    let tokens: Rc<RefCell<Vec<DisposableHandle>>> = Rc::default();
    let printed: Rc<RefCell<Vec<u32>>> = Rc::default();
    let output = Rc::clone(&printed);
    parser.set_print_handler(move |data| output.borrow_mut().extend_from_slice(data));
    for index in 0..3 {
        let tokens_weak = Rc::downgrade(&tokens);
        let token = parser
            .register_dcs_handler(
                FunctionIdentifier::new("q"),
                Rc::new(RefCell::new(DcsHandler::new(move |_, _| {
                    if index == 2 {
                        let tokens = tokens_weak.upgrade().unwrap();
                        for token in tokens.borrow().iter() {
                            token.dispose();
                        }
                    }
                    HandlerResult::Ready(false)
                }))),
            )
            .unwrap();
        tokens.borrow_mut().push(token);
    }
    let data: Vec<u32> = "\x1bPqpayload\u{9c}SUFFIX".chars().map(u32::from).collect();
    assert_eq!(
        parser.parse(&data, None).err(),
        Some(ParseError::Dispatch(DispatchError::ActiveHandlerRemoved))
    );
    assert!(printed.borrow().is_empty());
    parser.reset().unwrap();
    assert_eq!(parse(&mut parser, &[65]), 0);
    assert_eq!(*printed.borrow(), [65]);
}

#[test]
fn callback_failure_stops_suffix_and_reset_recovers_outer_parser() {
    let mut parser = EscapeSequenceParser::new();
    let printed: Rc<RefCell<Vec<u32>>> = Rc::default();
    let output = Rc::clone(&printed);
    parser.set_print_handler(move |data| output.borrow_mut().extend_from_slice(data));
    parser
        .register_dcs_handler(
            FunctionIdentifier::new("q"),
            Rc::new(RefCell::new(DcsHandler::new(|_, _| {
                HandlerResult::Failed(DispatchError::CallbackFailed)
            }))),
        )
        .unwrap();
    let data: Vec<u32> = "\x1bPqpayload\u{9c}SUFFIX".chars().map(u32::from).collect();
    assert_eq!(
        parser.parse(&data, None).err(),
        Some(ParseError::Dispatch(DispatchError::CallbackFailed))
    );
    assert!(printed.borrow().is_empty());
    parser.reset().unwrap();
    parser
        .clear_dcs_handler(FunctionIdentifier::new("q"))
        .unwrap();
    assert_eq!(parse(&mut parser, &[65]), 0);
    assert_eq!(*printed.borrow(), [65]);
}
