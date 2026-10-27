// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Assertions ported from common/parser/ApcParser.test.ts.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll, Wake, Waker};
use xterm_core::parser::apc::{ApcAction, ApcHandler, ApcHandlerTrait, ApcParser, HandlerHandle};
use xterm_core::parser::handler::HandlerResult;
use xterm_core::text_decoder::utf32_to_string;

const ID: u32 = 0x2b70;
type Reports = Rc<RefCell<Vec<String>>>;

struct NoWake;
impl Wake for NoWake {
    fn wake(self: Arc<Self>) {}
}

fn complete(parser: &mut ApcParser, success: bool) {
    let mut previous = true;
    while let Some(result) = parser.end(success, previous) {
        match result {
            HandlerResult::Ready(value) => previous = value,
            HandlerResult::Pending(mut future) => {
                let waker = Waker::from(Arc::new(NoWake));
                let mut context = Context::from_waker(&waker);
                // Test futures resolve on their first poll. Never hide stalled work.
                previous = match future.as_mut().poll(&mut context) {
                    Poll::Ready(value) => value,
                    Poll::Pending => panic!("test future did not resolve"),
                };
            }
        }
    }
}

struct Reporter {
    reports: Reports,
    name: &'static str,
    handled: bool,
    asynchronous: bool,
}
impl ApcHandlerTrait for Reporter {
    fn start(&mut self) {
        self.reports
            .borrow_mut()
            .push(format!("{} START", self.name));
    }
    fn put(&mut self, data: &[u32]) {
        self.reports.borrow_mut().push(format!(
            "{} PUT {}",
            self.name,
            String::from_utf16_lossy(&utf32_to_string(data))
        ));
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        let reports = Rc::clone(&self.reports);
        let name = self.name;
        let handled = self.handled;
        if self.asynchronous {
            HandlerResult::Pending(Box::pin(async move {
                reports.borrow_mut().push(format!("{name} END {success}"));
                handled
            }))
        } else {
            reports.borrow_mut().push(format!("{name} END {success}"));
            HandlerResult::Ready(handled)
        }
    }
}

fn register(
    parser: &mut ApcParser,
    reports: &Reports,
    name: &'static str,
    handled: bool,
    asynchronous: bool,
) -> xterm_core::lifecycle::DisposableHandle {
    let handler: HandlerHandle = Rc::new(RefCell::new(Reporter {
        reports: Rc::clone(reports),
        name,
        handled,
        asynchronous,
    }));
    parser.register_handler(ID, handler)
}
fn put(parser: &mut ApcParser, text: &str) {
    parser.put(&text.chars().map(u32::from).collect::<Vec<_>>());
}
fn run(parser: &mut ApcParser) {
    parser.start(ID);
    put(parser, "Here comes");
    put(parser, "the mouse!");
    complete(parser, true);
}
fn expected(names: &[&str], handled: bool) -> Vec<String> {
    let mut output = Vec::new();
    for name in names {
        output.push(format!("{name} START"));
    }
    for text in ["Here comes", "the mouse!"] {
        for name in names {
            output.push(format!("{name} PUT {text}"));
        }
    }
    for (index, name) in names.iter().enumerate() {
        output.push(format!("{name} END {}", !handled || index == 0));
    }
    output
}

#[test]
fn set_apc_handler() {
    let mut parser = ApcParser::new();
    let reports = Reports::default();
    register(&mut parser, &reports, "th", true, false);
    run(&mut parser);
    assert_eq!(*reports.borrow(), expected(&["th"], true));
}
#[test]
fn clear_apc_handler() {
    let mut parser = ApcParser::new();
    let reports = Reports::default();
    register(&mut parser, &reports, "th", true, false);
    parser.clear_handler(ID);
    let actual = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&actual);
    parser.set_handler_fallback(move |id, action| output.borrow_mut().push((id, action)));
    run(&mut parser);
    assert_eq!(
        *actual.borrow(),
        vec![
            (ID, ApcAction::Start),
            (ID, ApcAction::Put("Here comes".encode_utf16().collect())),
            (ID, ApcAction::Put("the mouse!".encode_utf16().collect())),
            (ID, ApcAction::End(true)),
        ]
    );
    assert!(reports.borrow().is_empty());
}
#[test]
fn add_apc_handler() {
    check_pair(true);
}
#[test]
fn add_apc_handler_return_false() {
    check_pair(false);
}
fn check_pair(handled: bool) {
    let mut parser = ApcParser::new();
    let reports = Reports::default();
    register(&mut parser, &reports, "th1", true, false);
    register(&mut parser, &reports, "th2", handled, false);
    run(&mut parser);
    assert_eq!(*reports.borrow(), expected(&["th2", "th1"], handled));
}
#[test]
fn dispose_handlers() {
    let mut parser = ApcParser::new();
    let reports = Reports::default();
    register(&mut parser, &reports, "th1", true, false);
    let token = register(&mut parser, &reports, "th2", false, false);
    token.dispose();
    token.dispose();
    run(&mut parser);
    assert_eq!(*reports.borrow(), expected(&["th1"], true));
}

#[test]
fn mixed_sync_async_dispatch() {
    // Four original cases: two stack layouts, first handles or all bubble.
    for asynchronous in [[false, true, false], [true, false, true]] {
        for handled in [true, false] {
            let names = if asynchronous[0] {
                ["a1", "s1", "a2"]
            } else {
                ["s1", "a1", "s2"]
            };
            let mut parser = ApcParser::new();
            let reports = Reports::default();
            for index in 0..3 {
                register(
                    &mut parser,
                    &reports,
                    names[index],
                    handled,
                    asynchronous[index],
                );
            }
            run(&mut parser);
            let reverse = [names[2], names[1], names[0]];
            assert_eq!(*reports.borrow(), expected(&reverse, handled));
        }
    }
}

type Calls = Rc<RefCell<Vec<(&'static str, Vec<u16>)>>>;
fn factory(
    parser: &mut ApcParser,
    calls: &Calls,
    name: &'static str,
    handled: bool,
    asynchronous: bool,
) -> xterm_core::lifecycle::DisposableHandle {
    let calls = Rc::clone(calls);
    let callback = move |data: Vec<u16>| {
        calls.borrow_mut().push((name, data));
        if asynchronous {
            HandlerResult::Pending(Box::pin(async move { handled }))
        } else {
            HandlerResult::Ready(handled)
        }
    };
    parser.register_handler(
        ID,
        Rc::new(RefCell::new(ApcHandler::with_limit(callback, 100))),
    )
}
fn payload(parser: &mut ApcParser, success: bool, first: &str, second: &str) {
    parser.start(ID);
    put(parser, first);
    put(parser, second);
    complete(parser, success);
}

#[test]
fn factory_end_success_and_abort_sync_async() {
    for asynchronous in [false, true] {
        for success in [false, true] {
            let mut parser = ApcParser::new();
            let calls = Calls::default();
            factory(&mut parser, &calls, "", true, asynchronous);
            payload(&mut parser, success, "Here comes", " the mouse!");
            let expected = if success {
                vec![("", "Here comes the mouse!".encode_utf16().collect())]
            } else {
                vec![]
            };
            assert_eq!(*calls.borrow(), expected);
        }
    }
}
#[test]
fn factory_dispose_sync_async() {
    for asynchronous in [false, true] {
        let mut parser = ApcParser::new();
        let calls = Calls::default();
        factory(&mut parser, &calls, "one", true, asynchronous);
        let token = factory(&mut parser, &calls, "two", true, asynchronous);
        payload(&mut parser, true, "Here comes", " the mouse!");
        assert_eq!(
            *calls.borrow(),
            vec![(
                "two",
                "Here comes the mouse!".encode_utf16().collect::<Vec<_>>()
            )]
        );
        token.dispose();
        payload(&mut parser, true, "some other", " data");
        assert_eq!(
            *calls.borrow(),
            vec![
                (
                    "two",
                    "Here comes the mouse!".encode_utf16().collect::<Vec<_>>()
                ),
                ("one", "some other data".encode_utf16().collect()),
            ]
        );
    }
}
#[test]
fn factory_return_false_sync_async() {
    for asynchronous in [false, true] {
        let mut parser = ApcParser::new();
        let calls = Calls::default();
        factory(&mut parser, &calls, "one", true, asynchronous);
        factory(&mut parser, &calls, "two", false, asynchronous);
        payload(&mut parser, true, "Here comes", " the mouse!");
        let text: Vec<_> = "Here comes the mouse!".encode_utf16().collect();
        assert_eq!(*calls.borrow(), vec![("two", text.clone()), ("one", text)]);
    }
}
#[test]
fn factory_payload_limit_and_overflow() {
    for overflow in [false, true] {
        let mut parser = ApcParser::new();
        let calls = Calls::default();
        factory(&mut parser, &calls, "", true, false);
        parser.start(ID);
        for _ in 0..10 {
            put(&mut parser, "AAAAAAAAAA");
        }
        if overflow {
            put(&mut parser, "A");
        }
        complete(&mut parser, true);
        let expected = if overflow {
            vec![]
        } else {
            vec![("", vec![65; 100])]
        };
        assert_eq!(*calls.borrow(), expected);
        payload(&mut parser, true, "clean", "");
        assert_eq!(
            calls.borrow().last(),
            Some(&("", "clean".encode_utf16().collect()))
        );
    }
}
#[test]
fn reset_aborts_active_handlers() {
    let mut parser = ApcParser::new();
    let reports = Reports::default();
    register(&mut parser, &reports, "th", true, false);
    parser.start(ID);
    put(&mut parser, "partial");
    parser.reset();
    assert_eq!(
        *reports.borrow(),
        ["th START", "th PUT partial", "th END false"]
    );
    reports.borrow_mut().clear();
    parser.start(ID);
    put(&mut parser, "complete");
    complete(&mut parser, true);
    assert_eq!(
        *reports.borrow(),
        ["th START", "th PUT complete", "th END true"]
    );
}
