// Copyright (c) 2018-2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. OSC integration via the real VT500 escape parser.
mod parser_support;
use parser_support::{report, units, Log, Reporter};
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Wake, Waker};
use xterm_core::parser::escape::{EscapeSequenceParser, ParseError};
use xterm_core::parser::handler::{DispatchError, HandlerResult};
#[path = "parser_support/removal.rs"]
mod removal;
use removal::RemoveAll;
#[path = "parser_support/future.rs"]
mod fixture_future;
use xterm_core::parser::osc::OscHandler;
use xterm_core::parser::osc::OscHandlerTrait;
use xterm_core::parser::params::Params;
use xterm_core::text_decoder::utf32_to_string;

struct NoWake;
impl Wake for NoWake {
    fn wake(self: std::sync::Arc<Self>) {}
}
fn resolve(mut future: Pin<Box<dyn Future<Output = bool>>>) -> bool {
    let waker = Waker::from(std::sync::Arc::new(NoWake));
    match future.as_mut().poll(&mut Context::from_waker(&waker)) {
        Poll::Ready(result) => result,
        Poll::Pending => panic!("fixture future must resolve immediately when polled"),
    }
}
struct Handler {
    inner: Reporter,
    asynchronous: bool,
}
impl OscHandlerTrait for Handler {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.inner.start()?;
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.inner.put(data)?;
        Ok(())
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        if self.asynchronous {
            let mut inner = self.inner.clone();
            HandlerResult::Pending(Box::pin(async move {
                let HandlerResult::Ready(value) = inner.end(success) else {
                    unreachable!("reporter is synchronous")
                };
                value
            }))
        } else {
            self.inner.end(success)
        }
    }
}
fn parameters(params: &Params) -> String {
    params
        .values()
        .iter()
        .map(i32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
fn configured(mode: usize, log: &Log) -> EscapeSequenceParser {
    let mut parser = EscapeSequenceParser::new();
    let output = Rc::clone(log);
    parser.set_print_handler(move |data| {
        output
            .borrow_mut()
            .push(format!("t:{}", units(&utf32_to_string(data))))
    });
    let output = Rc::clone(log);
    parser.set_execute_handler_fallback(move |code| output.borrow_mut().push(format!("c:{code}")));
    let output = Rc::clone(log);
    parser.set_esc_handler_fallback(move |id| output.borrow_mut().push(format!("e:{id}")));
    let output = Rc::clone(log);
    parser.set_csi_handler_fallback(move |id, params| {
        output
            .borrow_mut()
            .push(format!("i:{id}:[{}]", parameters(params)))
    });
    let output = Rc::clone(log);
    parser
        .osc
        .set_handler_fallback(move |id, action| report(&output, id, action));
    let modes: &[&[(bool, bool)]] = &[
        &[],
        &[(true, false)],
        &[(false, false), (false, true), (true, false)],
        &[(true, true), (false, false), (false, true)],
        &[(false, false), (false, true)],
    ];
    for (name, &(handled, asynchronous)) in modes[mode].iter().enumerate() {
        parser.osc.register_handler(
            2.0,
            Rc::new(RefCell::new(Handler {
                inner: Reporter {
                    name: name.to_string(),
                    handled,
                    log: Rc::clone(log),
                },
                asynchronous,
            })),
        );
    }
    parser
}
fn drain_cleanup(parser: &mut EscapeSequenceParser) {
    for result in parser.take_cleanup() {
        let HandlerResult::Pending(future) = result else {
            panic!("fixture cleanup is infallible pending")
        };
        resolve(future);
    }
}
fn parse(parser: &mut EscapeSequenceParser, chunk: &[u32], log: &Log, mut previous: Option<bool>) {
    while let Some(result) = parser.parse(chunk, previous).unwrap() {
        log.borrow_mut().push("w".into());
        drain_cleanup(parser);
        let HandlerResult::Pending(future) = result else {
            panic!("only pending futures interrupt parse")
        };
        previous = Some(resolve(future));
    }
    drain_cleanup(parser);
}
#[test]
fn every_osc_mutation_exception_propagates_through_outer_parser() {
    for phase in ["start", "put", "end", "reset"] {
        let tokens = Rc::new(RefCell::new(Vec::new()));
        let mut parser = EscapeSequenceParser::new();
        let handler = Rc::new(RefCell::new(RemoveAll {
            phase: if phase == "reset" { "end" } else { phase },
            tokens: Rc::downgrade(&tokens),
        }));
        for _ in 0..2 {
            tokens
                .borrow_mut()
                .push(parser.osc.register_handler(2.0, handler.clone()));
        }
        let input = [27, 93, 50, 59, 65, 7];
        let result = if phase == "reset" {
            assert!(parser.parse(&input[..5], None).unwrap().is_none());
            parser.reset()
        } else {
            parser.parse(&input, None).map(|_| ())
        };
        assert!(
            matches!(
                result,
                Err(ParseError::Dispatch(DispatchError::ActiveHandlerRemoved))
            ),
            "phase {phase}"
        );
    }
}
#[test]
fn failed_callback_stops_before_trailing_print_and_does_not_invoke_older_handler() {
    let log = Log::default();
    let mut parser = configured(0, &log);
    let output = Rc::clone(&log);
    parser.osc.register_handler(
        2.0,
        Rc::new(RefCell::new(OscHandler::new(move |_| {
            output.borrow_mut().push("older".into());
            HandlerResult::Ready(true)
        }))),
    );
    parser.osc.register_handler(
        2.0,
        Rc::new(RefCell::new(OscHandler::new(|_| {
            HandlerResult::Failed(DispatchError::CallbackFailed)
        }))),
    );
    assert!(matches!(
        parser.parse(&[27, 93, 50, 59, 65, 7, 89], None),
        Err(ParseError::Dispatch(DispatchError::CallbackFailed))
    ));
    assert!(log.borrow().is_empty());
    parser.reset().unwrap();
    assert!(parser.parse(&[90], None).unwrap().is_none());
    assert_eq!(*log.borrow(), ["t:5a"]);
}
#[test]
fn rejected_callback_future_keeps_parser_paused_until_explicit_reset() {
    let log = Log::default();
    let mut parser = configured(0, &log);
    parser.osc.register_handler(
        2.0,
        Rc::new(RefCell::new(OscHandler::new(|_| {
            HandlerResult::PendingFallible(Box::pin(async { Err(DispatchError::CallbackFailed) }))
        }))),
    );
    let input = [27, 93, 50, 59, 65, 7, 89];
    let result = parser.parse(&input, None).unwrap().unwrap();
    assert_eq!(
        fixture_future::resolve_checked(result),
        Err(DispatchError::CallbackFailed)
    );
    assert!(log.borrow().is_empty());
    parser.reset().unwrap();
    assert!(parser.parse(&input, Some(false)).unwrap().is_none());
    assert_eq!(*log.borrow(), ["t:59"]);
    assert!(fixture_future::resolve(HandlerResult::Ready(true)));
}
#[test]
fn every_osc_intro_terminator_and_chunk_boundary_matches_original_parser() {
    for line in include_str!("fixtures/escape-osc-oracle.tsv").lines() {
        let fields: Vec<_> = line.split('\t').collect();
        let mode = fields[1].parse().unwrap();
        let log = Log::default();
        let mut parser = configured(mode, &log);
        for operation in fields[2].split(';') {
            if operation == "x" {
                parser.reset().unwrap();
                drain_cleanup(&mut parser);
            } else {
                let chunk = operation
                    .split_once(':')
                    .unwrap()
                    .1
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|s| u32::from_str_radix(s, 16).unwrap())
                    .collect::<Vec<_>>();
                let previous = if operation.starts_with("r:") {
                    let Some(HandlerResult::Pending(future)) = parser.parse(&chunk, None).unwrap()
                    else {
                        panic!("reset requires pending handler")
                    };
                    log.borrow_mut().push("w".into());
                    parser.reset().unwrap();
                    let previous = resolve(future);
                    drain_cleanup(&mut parser);
                    Some(previous)
                } else {
                    None
                };
                parse(&mut parser, &chunk, &log, previous);
            }
            log.borrow_mut().push(format!(
                "s:{}:{}:{}:{}",
                parser.current_state as u8,
                parser.preceding_join_state.get(),
                parser.collect(),
                parameters(parser.params())
            ));
        }
        assert_eq!(
            log.borrow().join("|"),
            fields[3],
            "case {} operations {}",
            fields[0],
            fields[2]
        );
    }
}
