// Copyright (c) 2018-2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. APC differential tests through the actual escape parser.

use std::cell::RefCell;
use std::fmt::LowerHex;
use std::rc::Rc;
use xterm_core::parser::apc::{ApcAction, ApcHandlerTrait};
use xterm_core::parser::escape::{
    EscapeSequenceParser, FunctionIdentifier, ParseError, ParseStackState,
};
use xterm_core::parser::handler::{DispatchError, HandlerResult};
use xterm_core::parser::params::{ParamValue, Params};
use xterm_core::text_decoder::utf32_to_string;

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
        .map(|entry| match entry {
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
        .join(":")
}
fn snapshot(parser: &EscapeSequenceParser) -> String {
    format!(
        "{}/{}/{}/{}",
        parser.current_state as u8,
        parser.collect(),
        params(parser.params()),
        parser.preceding_join_state.get()
    )
}
struct Reporter {
    id: u32,
    name: String,
    asynchronous: bool,
    handled: bool,
    log: Log,
}
impl ApcHandlerTrait for Reporter {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.log
            .borrow_mut()
            .push(format!("H:{}:{}", self.id, self.name));
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.log.borrow_mut().push(format!(
            "D:{}:{}:{}",
            self.id,
            self.name,
            hex(&utf32_to_string(data))
        ));
        Ok(())
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        let log = Rc::clone(&self.log);
        let event = format!("U:{}:{}:{}", self.id, self.name, u8::from(success));
        let handled = self.handled;
        if self.asynchronous {
            HandlerResult::Pending(Box::pin(async move {
                log.borrow_mut().push(event);
                handled
            }))
        } else {
            log.borrow_mut().push(event);
            HandlerResult::Ready(handled)
        }
    }
}
#[path = "parser_support/future.rs"]
mod fixture_future;
use fixture_future::resolve;

fn parser(configuration: &str, log: &Log) -> EscapeSequenceParser {
    let mut parser = EscapeSequenceParser::new();
    let output = Rc::clone(log);
    parser.set_print_handler(move |data| output.borrow_mut().push(format!("P:{}", hex(data))));
    let output = Rc::clone(log);
    parser.set_execute_handler_fallback(move |code| output.borrow_mut().push(format!("X:{code}")));
    let output = Rc::clone(log);
    parser.set_esc_handler_fallback(move |id| output.borrow_mut().push(format!("E:{id}")));
    let output = Rc::clone(log);
    parser.set_csi_handler_fallback(move |id, value| {
        output
            .borrow_mut()
            .push(format!("C:{id}:{}", params(value)))
    });
    let output = Rc::clone(log);
    parser.apc.set_handler_fallback(move |id, action| {
        let action = match action {
            ApcAction::Start => "s".to_owned(),
            ApcAction::Put(data) => format!("p:{}", hex(&data)),
            ApcAction::End(success) => format!("e:{}", u8::from(success)),
        };
        output.borrow_mut().push(format!("F:{id}:{action}"));
    });
    if configuration != "-" {
        for (intermediates, final_byte) in [
            (None, "G"),
            (Some("+"), "G"),
            (Some(" +"), "G"),
            (None, "0"),
        ] {
            let identifier = FunctionIdentifier {
                prefix: None,
                intermediates,
                final_byte,
            };
            let id = identifier.packed(true).unwrap();
            for entry in configuration.split(',') {
                let fields: Vec<_> = entry.split(':').collect();
                parser
                    .register_apc_handler(
                        identifier,
                        Rc::new(RefCell::new(Reporter {
                            id,
                            name: fields[0].to_owned(),
                            asynchronous: fields[1] == "1",
                            handled: fields[2] == "1",
                            log: Rc::clone(log),
                        })),
                    )
                    .unwrap();
            }
        }
    }
    parser
}

#[test]
fn apc_control_chunk_and_async_traces_match_actual_javascript_escape_parser() {
    let corpus = include_str!("fixtures/apc-escape.tsv");
    assert_eq!(corpus.lines().count(), 280);
    let mut pauses = 0;
    for row in corpus.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let log = Log::default();
        let mut parser = parser(fields[1], &log);
        let mut snapshots = Vec::new();
        for operation in fields[2].split(';') {
            if operation == "x" {
                parser.reset().unwrap();
                for cleanup in parser.take_cleanup() {
                    resolve(cleanup);
                }
            } else {
                let data: Vec<_> = operation
                    .strip_prefix("p:")
                    .unwrap()
                    .split(',')
                    .filter(|s| !s.is_empty())
                    .map(|value| u32::from_str_radix(value, 16).unwrap())
                    .collect();
                let mut previous = None;
                while let Some(pending) = parser.parse(&data, previous).unwrap() {
                    log.borrow_mut().push(format!(
                        "Y:{}:{}",
                        parser.current_state as u8,
                        parser.pause_info().unwrap().chunk_position
                    ));
                    pauses += 1;
                    previous = Some(resolve(pending));
                }
            }
            snapshots.push(snapshot(&parser));
        }
        assert_eq!(
            snapshots.join("~"),
            fields[3],
            "state {} {}",
            fields[0],
            fields[1]
        );
        assert_eq!(
            log.borrow().join("|"),
            fields[4],
            "callbacks {} {}",
            fields[0],
            fields[1]
        );
    }
    assert_eq!(pauses, 165);
}

struct ControlReporter {
    name: &'static str,
    asynchronous: bool,
    fail: bool,
    log: Log,
}
impl ApcHandlerTrait for ControlReporter {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.log.borrow_mut().push(format!("{}:S", self.name));
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.log
            .borrow_mut()
            .push(format!("{}:P:{}", self.name, hex(&utf32_to_string(data))));
        Ok(())
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        let log = Rc::clone(&self.log);
        let event = format!("{}:E:{}", self.name, u8::from(success));
        let fail = self.fail;
        if self.asynchronous {
            HandlerResult::PendingFallible(Box::pin(async move {
                log.borrow_mut().push(event);
                if fail {
                    Err(DispatchError::CallbackFailed)
                } else {
                    Ok(true)
                }
            }))
        } else {
            log.borrow_mut().push(event);
            if fail {
                HandlerResult::Failed(DispatchError::CallbackFailed)
            } else {
                HandlerResult::Ready(true)
            }
        }
    }
}

#[test]
fn actual_parser_callback_errors_and_reset_during_pause_match_source() {
    let fixture = include_str!("fixtures/apc-escape-control.tsv");
    assert_eq!(fixture.lines().count(), 3);
    for row in fixture.lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let kind = fields[0];
        let log = Log::default();
        let mut parser = EscapeSequenceParser::new();
        let output = Rc::clone(&log);
        parser.set_print_handler(move |data| output.borrow_mut().push(format!("P:{}", hex(data))));
        for name in ["a", "b"] {
            parser
                .register_apc_handler(
                    FunctionIdentifier::new("G"),
                    Rc::new(RefCell::new(ControlReporter {
                        name,
                        asynchronous: kind != "throw-sync",
                        fail: name == "b" && kind.starts_with("throw"),
                        log: Rc::clone(&log),
                    })),
                )
                .unwrap();
        }
        let input = [27, 95, 71, 100, 97, 116, 97, 156, 90];
        let result = parser.parse(&input, None);
        if kind == "throw-sync" {
            assert!(matches!(
                result,
                Err(ParseError::Dispatch(DispatchError::CallbackFailed))
            ));
        } else {
            let pending = result.unwrap().unwrap();
            log.borrow_mut().push(format!(
                "Y:{}:{}",
                parser.current_state as u8,
                parser.pause_info().unwrap().chunk_position
            ));
            if kind == "reset-pending" {
                parser.reset().unwrap();
            }
            let resolved = fixture_future::resolve_checked(pending);
            if kind == "reset-pending" {
                for cleanup in parser.take_cleanup() {
                    resolve(cleanup);
                }
                assert!(parser
                    .parse(&input, Some(resolved.unwrap()))
                    .unwrap()
                    .is_none());
                assert_eq!(fields[3], "none");
            } else {
                assert_eq!(resolved, Err(DispatchError::CallbackFailed));
                assert_eq!(fields[3], "Error");
            }
        }
        let stack = if parser.parse_stack_state() == ParseStackState::None {
            "none"
        } else {
            "paused"
        };
        assert_eq!(
            format!("{}/{stack}", snapshot(&parser)),
            fields[1],
            "state {kind}"
        );
        assert_eq!(log.borrow().join("|"), fields[2], "callbacks {kind}");
    }
}
