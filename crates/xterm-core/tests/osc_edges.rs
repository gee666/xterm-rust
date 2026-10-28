// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Additional OSC boundary and cancellation assertions.
use std::cell::RefCell;
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, Wake, Waker};
#[path = "parser_support/removal.rs"]
mod removal;
use removal::RemoveAll;
use xterm_core::parser::handler::HandlerResult;
use xterm_core::parser::osc::{OscAction, OscError, OscHandler, OscHandlerTrait, OscParser};

struct NoWake;
impl Wake for NoWake {
    fn wake(self: std::sync::Arc<Self>) {}
}
fn poll(mut future: Pin<Box<dyn Future<Output = bool>>>) -> bool {
    let waker = Waker::from(std::sync::Arc::new(NoWake));
    match future.as_mut().poll(&mut Context::from_waker(&waker)) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("fixture must be ready"),
    }
}
#[test]
fn utf16_lone_surrogates_and_astral_units_count_toward_limit() {
    for limit in [2, 3] {
        let output = Rc::new(RefCell::new(Vec::new()));
        let log = Rc::clone(&output);
        let mut handler = OscHandler::with_limit(
            move |data| {
                log.borrow_mut().push(data);
                HandlerResult::Ready(true)
            },
            limit,
        );
        handler.start();
        handler.put(&[0x1f600, 0xd800]);
        let HandlerResult::Ready(result) = handler.end(true) else {
            panic!("sync")
        };
        assert_eq!(result, limit == 3);
        assert_eq!(
            *output.borrow(),
            if limit == 3 {
                vec![vec![0xd83d, 0xde00, 0xd800]]
            } else {
                vec![]
            }
        );
        handler.start();
        handler.put(&[65]);
        assert!(matches!(handler.end(true), HandlerResult::Ready(true)));
        assert_eq!(output.borrow().last(), Some(&vec![65]));
    }
}
#[test]
fn zero_limit_overflow_ignores_later_chunks_and_recovers_after_start() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&calls);
    let mut handler = OscHandler::with_limit(
        move |data| {
            log.borrow_mut().push(data);
            HandlerResult::Ready(true)
        },
        0,
    );
    handler.start();
    handler.put(&[]);
    assert!(matches!(handler.end(true), HandlerResult::Ready(true)));
    handler.start();
    handler.put(&[65]);
    handler.put(&[]);
    assert!(matches!(handler.end(true), HandlerResult::Ready(false)));
    handler.start();
    assert!(matches!(handler.end(true), HandlerResult::Ready(true)));
    assert_eq!(*calls.borrow(), [Vec::<u16>::new(), Vec::new()]);
}
#[test]
fn clearing_registry_does_not_replace_active_handler_list() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut parser = OscParser::new();
    let log = Rc::clone(&calls);
    let old = Rc::new(RefCell::new(OscHandler::new(move |data| {
        log.borrow_mut().push(("old", data));
        HandlerResult::Ready(true)
    })));
    parser.register_handler(1.0, old);
    parser.start().unwrap();
    parser.put(&[49, 59, 65]).unwrap();
    parser.clear_handler(1.0);
    let log = Rc::clone(&calls);
    parser.register_handler(
        1.0,
        Rc::new(RefCell::new(OscHandler::new(move |data| {
            log.borrow_mut().push(("new", data));
            HandlerResult::Ready(true)
        }))),
    );
    parser.put(&[66]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    parser.start().unwrap();
    parser.put(&[49, 59, 67]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*calls.borrow(), [("old", vec![65, 66]), ("new", vec![67])]);
}
#[test]
fn repeated_disposal_removes_next_equal_identity_registration() {
    let calls = Rc::new(std::cell::Cell::new(0));
    let log = Rc::clone(&calls);
    let mut parser = OscParser::new();
    let handler = Rc::new(RefCell::new(OscHandler::new(move |_| {
        log.set(log.get() + 1);
        HandlerResult::Ready(false)
    })));
    let token = parser.register_handler(1.0, handler.clone());
    let other = parser.register_handler(1.0, handler);
    token.dispose();
    parser.start().unwrap();
    parser.put(&[49, 59, 65]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(calls.get(), 1);
    token.dispose();
    parser.start().unwrap();
    parser.put(&[49, 59, 66]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(calls.get(), 1);
    other.dispose();
}
#[test]
fn empty_and_illegal_identifier_boundaries() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&output);
    let mut parser = OscParser::new();
    parser.set_handler_fallback(move |id, action| log.borrow_mut().push((id, action)));
    parser.start().unwrap();
    parser.put(&[59]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(
        *output.borrow(),
        [(-1.0, OscAction::Start), (-1.0, OscAction::End(true))]
    );
    output.borrow_mut().clear();
    parser.start().unwrap();
    parser.put(&[49, 65, 59, 66]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert!(output.borrow().is_empty());
    parser.start().unwrap();
    assert!(parser.end(false, true).unwrap().is_none());
    assert_eq!(
        *output.borrow(),
        [(-1.0, OscAction::Start), (-1.0, OscAction::End(false))]
    );
}
#[test]
fn numeric_identifier_preserves_javascript_float_rounding_and_overflow() {
    let output = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&output);
    let mut parser = OscParser::new();
    parser.set_handler_fallback(move |id, _| log.borrow_mut().push(id));
    for digits in ["9007199254740993".to_owned(), "9".repeat(320)] {
        parser.start().unwrap();
        parser
            .put(&digits.chars().map(u32::from).collect::<Vec<_>>())
            .unwrap();
        assert!(parser.end(true, true).unwrap().is_none());
    }
    // The source computes multiply + ASCII digit - 48, rather than adding the
    // converted digit; rounding at this boundary gives 9007199254740992.
    assert_eq!(output.borrow()[0], 9_007_199_254_740_992.0);
    assert_eq!(output.borrow()[2], f64::INFINITY);
}
#[test]
fn live_removal_during_every_callback_stage_returns_recoverable_error() {
    for phase in ["start", "put", "end", "reset"] {
        let tokens = Rc::new(RefCell::new(Vec::new()));
        let mut parser = OscParser::new();
        let handler = Rc::new(RefCell::new(RemoveAll {
            phase: if phase == "reset" { "end" } else { phase },
            tokens: Rc::downgrade(&tokens),
        }));
        for _ in 0..2 {
            tokens
                .borrow_mut()
                .push(parser.register_handler(1.0, handler.clone()));
        }
        parser.start().unwrap();
        let put_result = parser.put(&[49, 59, 65]);
        let result = match phase {
            "start" | "put" => put_result,
            "end" => {
                put_result.unwrap();
                parser.end(true, true).map(|_| ())
            }
            "reset" => {
                put_result.unwrap();
                parser.reset()
            }
            _ => unreachable!(),
        };
        assert_eq!(result, Err(OscError::ActiveHandlerRemoved), "phase {phase}");
    }
}

struct AsyncHandler {
    name: u8,
    log: Rc<RefCell<Vec<(u8, bool)>>>,
}
impl OscHandlerTrait for AsyncHandler {
    fn start(&mut self) {}
    fn put(&mut self, _: &[u32]) {}
    fn end(&mut self, success: bool) -> HandlerResult {
        let name = self.name;
        let log = Rc::clone(&self.log);
        HandlerResult::Pending(Box::pin(async move {
            log.borrow_mut().push((name, success));
            true
        }))
    }
}
#[test]
fn reset_while_paused_cleans_only_lower_handlers_and_preserves_owned_future() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut parser = OscParser::new();
    for name in [1, 2, 3] {
        parser.register_handler(
            1.0,
            Rc::new(RefCell::new(AsyncHandler {
                name,
                log: Rc::clone(&log),
            })),
        );
    }
    parser.start().unwrap();
    parser.put(&[49, 59, 65]).unwrap();
    let Some(HandlerResult::Pending(first)) = parser.end(true, true).unwrap() else {
        panic!("pending")
    };
    parser.reset().unwrap();
    assert!(poll(first));
    for result in parser.take_cleanup() {
        let HandlerResult::Pending(future) = result else {
            panic!("pending")
        };
        assert!(poll(future));
    }
    assert_eq!(*log.borrow(), [(3, true), (2, false), (1, false)]);
    assert!(parser.end(true, true).unwrap().is_none());
}
