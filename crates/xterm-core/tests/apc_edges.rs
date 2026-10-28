// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Additional lifecycle and UTF-16 boundary checks.

use std::cell::RefCell;
use std::rc::Rc;
use xterm_core::parser::apc::{ApcAction, ApcHandler, ApcHandlerTrait, ApcParser};
use xterm_core::parser::handler::{DispatchError, HandlerResult};

type Log = Rc<RefCell<Vec<(u8, bool)>>>;
struct Async {
    id: u8,
    log: Log,
    handled: bool,
}
impl ApcHandlerTrait for Async {
    fn start(&mut self) -> Result<(), DispatchError> {
        Ok(())
    }
    fn put(&mut self, _: &[u32]) -> Result<(), DispatchError> {
        Ok(())
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        let log = Rc::clone(&self.log);
        let id = self.id;
        let handled = self.handled;
        HandlerResult::Pending(Box::pin(async move {
            log.borrow_mut().push((id, success));
            handled
        }))
    }
}
#[path = "parser_support/future.rs"]
mod fixture_future;
use fixture_future::resolve;

fn register(parser: &mut ApcParser, id: u8, log: &Log) {
    parser.register_handler(
        71,
        Rc::new(RefCell::new(Async {
            id,
            log: Rc::clone(log),
            handled: true,
        })),
    );
}

#[test]
fn reset_retains_lazy_abort_cleanup_through_disposal_in_reverse_order() {
    let log = Log::default();
    let mut parser = ApcParser::new();
    register(&mut parser, 1, &log);
    register(&mut parser, 2, &log);
    parser.start(71).unwrap();
    parser.reset().unwrap();
    parser.dispose();
    let cleanup = parser.take_cleanup();
    assert_eq!(cleanup.len(), 2);
    for future in cleanup {
        assert!(resolve(future));
    }
    assert_eq!(*log.borrow(), [(2, false), (1, false)]);
    assert!(parser.take_cleanup().is_empty());
}
#[test]
fn reset_while_paused_does_not_abort_the_already_returned_future() {
    let log = Log::default();
    let mut parser = ApcParser::new();
    register(&mut parser, 1, &log);
    register(&mut parser, 2, &log);
    parser.start(71).unwrap();
    let pending = parser.end(true, true).unwrap().unwrap();
    parser.reset().unwrap();
    assert!(resolve(pending));
    let cleanup = parser.take_cleanup();
    assert_eq!(cleanup.len(), 1);
    for future in cleanup {
        assert!(resolve(future));
    }
    assert_eq!(*log.borrow(), [(2, true), (1, false)]);
}
#[test]
fn idle_calls_and_disposal_use_identifier_zero_without_start_notifications() {
    let mut parser = ApcParser::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&log);
    parser.set_handler_fallback(move |id, action| output.borrow_mut().push((id, action)));
    parser.put(&[]).unwrap();
    assert!(parser.end(false, true).unwrap().is_none());
    assert_eq!(
        *log.borrow(),
        [(0, ApcAction::Put(vec![])), (0, ApcAction::End(false))]
    );
    parser.dispose();
    parser.start(71).unwrap();
    parser.put(&[65]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(log.borrow().len(), 2);
}
#[test]
fn payload_limit_counts_utf16_units_and_preserves_lone_surrogates() {
    let mut parser = ApcParser::new();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    parser.register_handler(
        71,
        Rc::new(RefCell::new(ApcHandler::with_limit(
            move |data| {
                output.borrow_mut().push(data);
                HandlerResult::Ready(true)
            },
            3,
        ))),
    );
    parser.start(71).unwrap();
    parser.put(&[0x1f600, 0xd800]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*calls.borrow(), [vec![0xd83d, 0xde00, 0xd800]]);
    parser.start(71).unwrap();
    parser.put(&[0x1f600, 0x1f600]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(calls.borrow().len(), 1);
}
#[test]
fn clearing_a_registration_does_not_abort_an_active_command() {
    let log = Log::default();
    let mut parser = ApcParser::new();
    register(&mut parser, 1, &log);
    parser.start(71).unwrap();
    parser.clear_handler(71);
    let pending = parser.end(true, true).unwrap().unwrap();
    let handled = resolve(pending);
    assert!(parser.end(true, handled).unwrap().is_none());
    assert_eq!(*log.borrow(), [(1, true)]);
    parser.start(71).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(log.borrow().len(), 1);
}

#[test]
fn asynchronous_false_during_cleanup_does_not_restore_bubbling() {
    let log = Log::default();
    let mut parser = ApcParser::new();
    for (id, handled) in [(1, true), (2, false), (3, true)] {
        parser.register_handler(
            71,
            Rc::new(RefCell::new(Async {
                id,
                log: Rc::clone(&log),
                handled,
            })),
        );
    }
    parser.start(71).unwrap();
    let mut previous = true;
    while let Some(result) = parser.end(true, previous).unwrap() {
        previous = resolve(result);
    }
    assert_eq!(*log.borrow(), [(3, true), (2, false), (1, false)]);
}

#[test]
fn dropped_callback_future_does_not_leak_payload_into_next_command() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let mut handler = ApcHandler::new(move |data| {
        output.borrow_mut().push(data);
        HandlerResult::Pending(Box::pin(async { true }))
    });
    handler.start().unwrap();
    handler.put(&[65, 66]).unwrap();
    drop(handler.end(true));
    handler.start().unwrap();
    handler.put(&[67]).unwrap();
    assert!(resolve(handler.end(true)));
    assert_eq!(*calls.borrow(), [vec![65, 66], vec![67]]);
    handler.start().unwrap();
    assert!(resolve(handler.end(true)));
    assert_eq!(calls.borrow().last(), Some(&vec![]));
    assert_eq!(xterm_core::parser::apc::PAYLOAD_LIMIT, 10_000_000);
}

#[test]
fn callback_failure_does_not_bubble_as_false_to_older_handlers() {
    let log = Log::default();
    let mut parser = ApcParser::new();
    register(&mut parser, 1, &log);
    parser.register_handler(
        71,
        Rc::new(RefCell::new(ApcHandler::new(|_| {
            HandlerResult::Failed(DispatchError::CallbackFailed)
        }))),
    );
    parser.start(71).unwrap();
    assert_eq!(
        parser.end(true, true).err(),
        Some(DispatchError::CallbackFailed)
    );
    assert!(log.borrow().is_empty());
    parser.reset().unwrap();
    for cleanup in parser.take_cleanup() {
        resolve(cleanup);
    }
    assert_eq!(*log.borrow(), [(1, false)]);
}

#[test]
fn rejected_callback_future_preserves_pause_until_explicit_reset() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let mut parser = ApcParser::new();
    parser.register_handler(
        71,
        Rc::new(RefCell::new(ApcHandler::new(move |data| {
            output.borrow_mut().push(data);
            if output.borrow().len() == 1 {
                HandlerResult::PendingFallible(Box::pin(async {
                    Err(DispatchError::CallbackFailed)
                }))
            } else {
                HandlerResult::Ready(true)
            }
        }))),
    );
    parser.start(71).unwrap();
    parser.put(&[65, 66]).unwrap();
    let pending = parser.end(true, true).unwrap().unwrap();
    assert_eq!(
        fixture_future::resolve_checked(pending),
        Err(DispatchError::CallbackFailed)
    );
    parser.reset().unwrap();
    assert!(parser.take_cleanup().is_empty());
    parser.start(71).unwrap();
    parser.put(&[67]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*calls.borrow(), [vec![65, 66], vec![67]]);
}

#[test]
fn checked_fallback_errors_propagate_without_losing_active_or_idle_identifiers() {
    for phase in 0..5 {
        let mut parser = ApcParser::new();
        let failing_action = if phase < 3 { phase } else { phase - 2 };
        parser.set_handler_fallback_checked(move |_, action| {
            let action = match action {
                ApcAction::Start => 0,
                ApcAction::Put(_) => 1,
                ApcAction::End(_) => 2,
            };
            if action == failing_action {
                Err(DispatchError::CallbackFailed)
            } else {
                Ok(())
            }
        });
        let result = match phase {
            0 => parser.start(71),
            1 => parser.start(71).and_then(|()| parser.put(&[65])),
            2 => parser
                .start(71)
                .and_then(|()| parser.end(true, true).map(|_| ())),
            3 => parser.put(&[65]),
            _ => parser.end(true, true).map(|_| ()),
        };
        assert_eq!(result, Err(DispatchError::CallbackFailed), "phase {phase}");
        let log = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&log);
        parser.set_handler_fallback(move |id, action| output.borrow_mut().push((id, action)));
        parser.put(&[66]).unwrap();
        assert_eq!(
            *log.borrow(),
            [(if phase < 3 { 71 } else { 0 }, ApcAction::Put(vec![66]))]
        );
    }
}
