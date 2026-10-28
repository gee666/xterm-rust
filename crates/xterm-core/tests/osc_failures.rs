// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Fallible Rust callback equivalents of source exceptions.
#[path = "parser_support/future.rs"]
mod future;
use std::cell::RefCell;
use std::rc::Rc;
use xterm_core::parser::handler::{DispatchError, HandlerResult};
use xterm_core::parser::osc::{OscHandler, OscHandlerTrait, OscParser};

#[test]
fn event_callback_cause_is_not_replaced_by_a_generic_error() {
    let cause =
        xterm_core::event::EmitError::callback(std::io::Error::other("exact listener failure"));
    let expected = cause.clone();
    let result = HandlerResult::from(Err(cause));
    let HandlerResult::Failed(DispatchError::Callback(actual)) = result else {
        panic!("typed cause must be retained")
    };
    assert_eq!(actual, expected);
    assert_eq!(
        actual.callback_ref::<std::io::Error>().unwrap().to_string(),
        "exact listener failure"
    );
}
#[test]
fn buffered_callback_failure_preserves_data_until_later_end_or_start() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&calls);
    let mut handler = OscHandler::new(move |data| {
        let mut log = log.borrow_mut();
        log.push(data);
        if log.len() == 1 {
            HandlerResult::Failed(DispatchError::CallbackFailed)
        } else {
            HandlerResult::Ready(true)
        }
    });
    handler.start();
    handler.put(&[65, 0x1f600, 0xd800]);
    assert!(matches!(
        handler.end(true),
        HandlerResult::Failed(DispatchError::CallbackFailed)
    ));
    assert!(matches!(handler.end(true), HandlerResult::Ready(true)));
    assert_eq!(
        *calls.borrow(),
        [
            vec![65, 0xd83d, 0xde00, 0xd800],
            vec![65, 0xd83d, 0xde00, 0xd800]
        ]
    );
    handler.start();
    handler.put(&[66]);
    assert!(matches!(handler.end(true), HandlerResult::Ready(true)));
    assert_eq!(calls.borrow().last(), Some(&vec![66]));
}
#[test]
fn rejected_future_retains_payload_and_successful_future_clears_it() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let log = Rc::clone(&calls);
    let mut handler = OscHandler::new(move |data| {
        let mut log = log.borrow_mut();
        log.push(data);
        let fail = log.len() == 1;
        HandlerResult::PendingFallible(Box::pin(async move {
            if fail {
                Err(DispatchError::CallbackFailed)
            } else {
                Ok(true)
            }
        }))
    });
    handler.start();
    handler.put(&[65]);
    assert_eq!(
        future::resolve_checked(handler.end(true)),
        Err(DispatchError::CallbackFailed)
    );
    assert!(future::resolve(handler.end(true)));
    assert!(future::resolve(handler.end(true)));
    assert_eq!(*calls.borrow(), [vec![65], vec![65], vec![]]);
}
#[test]
fn rejected_future_does_not_resume_bubbling_and_reset_cleans_older_handlers() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut parser = OscParser::new();
    parser.register_handler(
        2.0,
        Rc::new(RefCell::new(Failing {
            name: 1,
            fail: false,
            log: Rc::clone(&log),
        })),
    );
    parser.register_handler(
        2.0,
        Rc::new(RefCell::new(OscHandler::new(|_| {
            HandlerResult::PendingFallible(Box::pin(async { Err(DispatchError::CallbackFailed) }))
        }))),
    );
    parser.start().unwrap();
    parser.put(&[50, 59, 65]).unwrap();
    let pending = parser.end(true, true).unwrap().unwrap();
    assert_eq!(
        future::resolve_checked(pending),
        Err(DispatchError::CallbackFailed)
    );
    assert!(log.borrow().is_empty());
    parser.reset().unwrap();
    assert_eq!(*log.borrow(), [(1, false)]);
}

struct Failing {
    name: u8,
    fail: bool,
    log: Rc<RefCell<Vec<(u8, bool)>>>,
}
impl OscHandlerTrait for Failing {
    fn start(&mut self) {}
    fn put(&mut self, _: &[u32]) {}
    fn end(&mut self, success: bool) -> HandlerResult {
        self.log.borrow_mut().push((self.name, success));
        if self.fail {
            HandlerResult::Failed(DispatchError::CallbackFailed)
        } else {
            HandlerResult::Ready(false)
        }
    }
}
#[test]
fn failed_end_does_not_bubble_or_cleanup_lower_handlers() {
    for reset in [false, true] {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut parser = OscParser::new();
        for name in [1, 2] {
            parser.register_handler(
                2.0,
                Rc::new(RefCell::new(Failing {
                    name,
                    fail: name == 2,
                    log: Rc::clone(&log),
                })),
            );
        }
        parser.start().unwrap();
        parser.put(&[50, 59, 65]).unwrap();
        let result = if reset {
            parser.reset()
        } else {
            parser.end(true, true).map(|_| ())
        };
        assert_eq!(result, Err(DispatchError::CallbackFailed));
        assert_eq!(*log.borrow(), [(2, !reset)]);
    }
}
#[test]
fn cleanup_failure_stops_before_older_handlers() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let mut parser = OscParser::new();
    for name in [1, 2] {
        parser.register_handler(
            2.0,
            Rc::new(RefCell::new(Failing {
                name,
                fail: name == 2,
                log: Rc::clone(&log),
            })),
        );
    }
    parser.register_handler(
        2.0,
        Rc::new(RefCell::new(OscHandler::new(|_| {
            HandlerResult::Ready(true)
        }))),
    );
    parser.start().unwrap();
    parser.put(&[50, 59, 65]).unwrap();
    assert!(matches!(
        parser.end(true, true),
        Err(DispatchError::CallbackFailed)
    ));
    assert_eq!(*log.borrow(), [(2, false)]);
}
