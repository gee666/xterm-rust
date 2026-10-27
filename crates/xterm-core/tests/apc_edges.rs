// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Additional lifecycle and UTF-16 boundary checks.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use xterm_core::parser::apc::{ApcAction, ApcHandler, ApcHandlerTrait, ApcParser};
use xterm_core::parser::handler::HandlerResult;

type Log = Rc<RefCell<Vec<(u8, bool)>>>;
struct Async {
    id: u8,
    log: Log,
    handled: bool,
}
impl ApcHandlerTrait for Async {
    fn start(&mut self) {}
    fn put(&mut self, _: &[u32]) {}
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
struct NoWake;
impl Wake for NoWake {
    fn wake(self: Arc<Self>) {}
}
fn resolve(result: HandlerResult) -> bool {
    match result {
        HandlerResult::Ready(result) => result,
        HandlerResult::Pending(mut future) => {
            let waker = Waker::from(Arc::new(NoWake));
            match future.as_mut().poll(&mut Context::from_waker(&waker)) {
                Poll::Ready(value) => value,
                Poll::Pending => panic!("test future did not resolve"),
            }
        }
    }
}
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
    handler.start();
    handler.put(&[65, 66]);
    drop(handler.end(true));
    handler.start();
    handler.put(&[67]);
    assert!(resolve(handler.end(true)));
    assert_eq!(*calls.borrow(), [vec![65, 66], vec![67]]);
    handler.start();
    assert!(resolve(handler.end(true)));
    assert_eq!(calls.borrow().last(), Some(&vec![]));
    assert_eq!(xterm_core::parser::apc::PAYLOAD_LIMIT, 10_000_000);
}
