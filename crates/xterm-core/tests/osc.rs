// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Ported assertions from common/parser/OscParser.test.ts.
use std::cell::RefCell;
use std::rc::Rc;
#[path = "parser_support/future.rs"]
mod future;
use future::resolve;
use xterm_core::parser::handler::{DispatchError, HandlerResult};
use xterm_core::parser::osc::{HandlerHandle, OscAction, OscHandler, OscHandlerTrait, OscParser};

type Log = Rc<RefCell<Vec<String>>>;
fn utf32(value: &str) -> Vec<u32> {
    value.chars().map(u32::from).collect()
}
fn string(value: &[u16]) -> String {
    String::from_utf16(value).unwrap()
}
fn parser() -> (OscParser, Log) {
    let log = Log::default();
    let output = Rc::clone(&log);
    let mut parser = OscParser::new();
    parser.set_handler_fallback(move |id, action| {
        output.borrow_mut().push(match action {
            OscAction::Start => format!("f:{id}:s"),
            OscAction::Put(data) => format!("f:{id}:p:{}", string(&data)),
            OscAction::End(success) => format!("f:{id}:e:{success}"),
        });
    });
    (parser, log)
}
fn end(parser: &mut OscParser, success: bool) {
    let mut previous = true;
    while let Some(result) = parser.end(success, previous).unwrap() {
        previous = resolve(result);
    }
}
struct Handler {
    name: &'static str,
    log: Log,
    handled: bool,
    asynchronous: bool,
}
impl OscHandlerTrait for Handler {
    fn start(&mut self) -> Result<(), DispatchError> {
        self.log.borrow_mut().push(format!("{}:s", self.name));
        Ok(())
    }
    fn put(&mut self, data: &[u32]) -> Result<(), DispatchError> {
        self.log.borrow_mut().push(format!(
            "{}:p:{}",
            self.name,
            string(&xterm_core::text_decoder::utf32_to_string(data))
        ));
        Ok(())
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        let name = self.name;
        let log = Rc::clone(&self.log);
        let handled = self.handled;
        if self.asynchronous {
            HandlerResult::Pending(Box::pin(async move {
                log.borrow_mut().push(format!("{name}:e:{success}"));
                handled
            }))
        } else {
            log.borrow_mut().push(format!("{name}:e:{success}"));
            HandlerResult::Ready(handled)
        }
    }
}
fn handler(name: &'static str, log: &Log, handled: bool, asynchronous: bool) -> HandlerHandle {
    Rc::new(RefCell::new(Handler {
        name,
        log: Rc::clone(log),
        handled,
        asynchronous,
    }))
}
fn command(parser: &mut OscParser) {
    parser.start().unwrap();
    parser.put(&utf32("1234;Here comes")).unwrap();
    parser.put(&utf32("the mouse!")).unwrap();
    end(parser, true);
}

#[test]
fn identifier_parsing() {
    let (mut parser, log) = parser();
    parser.put(&utf32("hello world!")).unwrap();
    end(&mut parser, true);
    assert!(log.borrow().is_empty());
    parser.start().unwrap();
    parser.put(&utf32("12")).unwrap();
    parser.put(&utf32("34")).unwrap();
    end(&mut parser, true);
    assert_eq!(*log.borrow(), ["f:1234:s", "f:1234:e:true"]);
    log.borrow_mut().clear();
    parser.start().unwrap();
    for part in ["12", "34", ";h", "ello"] {
        parser.put(&utf32(part)).unwrap();
    }
    end(&mut parser, true);
    assert_eq!(
        *log.borrow(),
        ["f:1234:s", "f:1234:p:h", "f:1234:p:ello", "f:1234:e:true"]
    );
}

#[test]
fn registration_clear_and_disposal() {
    for action in ["registered", "cleared", "disposed"] {
        let (mut parser, log) = parser();
        let token = parser.register_handler(1234.0, handler("th", &log, true, false));
        if action == "cleared" {
            parser.clear_handler(1234.0);
        }
        if action == "disposed" {
            token.dispose();
            token.dispose();
        }
        command(&mut parser);
        let prefix = if action == "registered" {
            "th"
        } else {
            "f:1234"
        };
        assert_eq!(
            *log.borrow(),
            [
                format!("{prefix}:s"),
                format!("{prefix}:p:Here comes"),
                format!("{prefix}:p:the mouse!"),
                format!("{prefix}:e:true")
            ]
        );
    }
}

#[test]
fn bubbling_and_mixed_sync_async_cleanup() {
    for asynchronous in [[false, false], [false, true], [true, false], [true, true]] {
        for handled in [true, false] {
            let (mut parser, log) = parser();
            parser.register_handler(1234.0, handler("th1", &log, true, asynchronous[0]));
            parser.register_handler(1234.0, handler("th2", &log, handled, asynchronous[1]));
            command(&mut parser);
            assert_eq!(
                *log.borrow(),
                [
                    "th2:s",
                    "th1:s",
                    "th2:p:Here comes",
                    "th1:p:Here comes",
                    "th2:p:the mouse!",
                    "th1:p:the mouse!",
                    "th2:e:true",
                    if handled { "th1:e:false" } else { "th1:e:true" }
                ]
            );
        }
    }
    for flags in [[false, true, false], [true, false, true]] {
        for handled in [false, true] {
            let (mut parser, log) = parser();
            for (name, asynchronous) in ["s1", "a1", "s2"].into_iter().zip(flags) {
                parser.register_handler(1234.0, handler(name, &log, handled, asynchronous));
            }
            command(&mut parser);
            assert_eq!(
                *log.borrow(),
                [
                    "s2:s",
                    "a1:s",
                    "s1:s",
                    "s2:p:Here comes",
                    "a1:p:Here comes",
                    "s1:p:Here comes",
                    "s2:p:the mouse!",
                    "a1:p:the mouse!",
                    "s1:p:the mouse!",
                    "s2:e:true",
                    if handled { "a1:e:false" } else { "a1:e:true" },
                    if handled { "s1:e:false" } else { "s1:e:true" }
                ]
            );
        }
    }
}

fn factory(log: &Log, name: &'static str, handled: bool, asynchronous: bool) -> HandlerHandle {
    let log = Rc::clone(log);
    Rc::new(RefCell::new(OscHandler::with_limit(
        move |data| {
            log.borrow_mut().push(format!("{name}:{}", string(&data)));
            if asynchronous {
                HandlerResult::Pending(Box::pin(async move { handled }))
            } else {
                HandlerResult::Ready(handled)
            }
        },
        100,
    )))
}
#[test]
fn factory_success_abort_dispose_and_bubble() {
    for asynchronous in [false, true] {
        for success in [false, true] {
            let (mut parser, log) = parser();
            parser.register_handler(1234.0, factory(&log, "1234", true, asynchronous));
            parser.start().unwrap();
            parser.put(&utf32("1234;Here comes")).unwrap();
            parser.put(&utf32(" the mouse!")).unwrap();
            end(&mut parser, success);
            assert_eq!(
                *log.borrow(),
                if success {
                    vec!["1234:Here comes the mouse!"]
                } else {
                    vec![]
                }
            );
        }
        for handled in [true, false] {
            let (mut parser, log) = parser();
            parser.register_handler(1234.0, factory(&log, "one", true, asynchronous));
            let token =
                parser.register_handler(1234.0, factory(&log, "two", handled, asynchronous));
            parser.start().unwrap();
            parser.put(&utf32("1234;Here comes")).unwrap();
            parser.put(&utf32(" the mouse!")).unwrap();
            end(&mut parser, true);
            let mut expected = vec!["two:Here comes the mouse!"];
            if !handled {
                expected.push("one:Here comes the mouse!");
            }
            assert_eq!(*log.borrow(), expected);
            token.dispose();
            parser.start().unwrap();
            parser.put(&utf32("1234;some other")).unwrap();
            parser.put(&utf32(" data")).unwrap();
            end(&mut parser, true);
            expected.push("one:some other data");
            assert_eq!(*log.borrow(), expected);
        }
    }
}
#[test]
fn factory_exact_limit_and_limit_plus_one() {
    for asynchronous in [false, true] {
        for excess in [false, true] {
            let (mut parser, log) = parser();
            parser.register_handler(1234.0, factory(&log, "1234", true, asynchronous));
            parser.start().unwrap();
            parser.put(&utf32("1234;")).unwrap();
            for _ in 0..10 {
                parser.put(&utf32("AAAAAAAAAA")).unwrap();
            }
            if excess {
                parser.put(&utf32("A")).unwrap();
            }
            end(&mut parser, true);
            assert_eq!(
                *log.borrow(),
                if excess {
                    vec![]
                } else {
                    vec![format!("1234:{}", "A".repeat(100))]
                }
            );
        }
    }
}
#[test]
fn reset_payload_and_resume_after_reset() {
    let (mut parser, log) = parser();
    parser.register_handler(1234.0, handler("th", &log, true, false));
    parser.start().unwrap();
    parser.put(&utf32("1234;partial")).unwrap();
    parser.reset().unwrap();
    assert_eq!(*log.borrow(), ["th:s", "th:p:partial", "th:e:false"]);
    log.borrow_mut().clear();
    parser.start().unwrap();
    parser.put(&utf32("1234;complete")).unwrap();
    end(&mut parser, true);
    assert_eq!(*log.borrow(), ["th:s", "th:p:complete", "th:e:true"]);
}
#[test]
fn reset_retains_async_cleanup_futures() {
    let (mut parser, log) = parser();
    parser.register_handler(1234.0, handler("async", &log, true, true));
    parser.start().unwrap();
    parser.put(&utf32("1234;partial")).unwrap();
    parser.reset().unwrap();
    let cleanup = parser.take_cleanup();
    assert_eq!(cleanup.len(), 1);
    for result in cleanup {
        assert!(resolve(result));
    }
    assert_eq!(
        *log.borrow(),
        ["async:s", "async:p:partial", "async:e:false"]
    );
    assert!(parser.take_cleanup().is_empty());
}
