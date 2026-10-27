// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Mutable-list lifecycle and upstream error regressions.

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use xterm_core::lifecycle::DisposableHandle;
use xterm_core::parser::apc::{ApcAction, ApcHandler, ApcHandlerTrait, ApcParser};
use xterm_core::parser::handler::{DispatchError, HandlerResult};

type Log = Rc<RefCell<Vec<String>>>;
type Tokens = RefCell<Vec<DisposableHandle>>;
struct Mutation {
    name: &'static str,
    stage: String,
    log: Log,
    tokens: Weak<Tokens>,
}
impl Mutation {
    fn remove(&self) {
        if let Some(tokens) = self.tokens.upgrade() {
            for token in
                tokens
                    .borrow()
                    .iter()
                    .take(if self.stage == "shift" { 1 } else { usize::MAX })
            {
                token.dispose();
            }
        }
    }
}
impl ApcHandlerTrait for Mutation {
    fn start(&mut self) {
        self.log.borrow_mut().push(format!("{}:s", self.name));
        if self.name == "b" && ["start", "shift"].contains(&self.stage.as_str()) {
            self.remove();
        }
    }
    fn put(&mut self, _: &[u32]) {
        self.log.borrow_mut().push(format!("{}:p", self.name));
        if self.name == "b" && self.stage == "put" {
            self.remove();
        }
    }
    fn end(&mut self, success: bool) -> HandlerResult {
        self.log
            .borrow_mut()
            .push(format!("{}:e:{}", self.name, u8::from(success)));
        if self.name == "b" && ["end", "reset"].contains(&self.stage.as_str()) {
            self.remove();
        }
        HandlerResult::Ready(false)
    }
}

#[test]
fn callback_removal_errors_match_upstream_partial_logs_in_all_dispatch_phases() {
    for row in include_str!("fixtures/apc-mutation.tsv").lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let stage = fields[0];
        let log = Log::default();
        let tokens = Rc::new(Tokens::default());
        let mut parser = ApcParser::new();
        for name in ["a", "b"] {
            let token = parser.register_handler(
                71,
                Rc::new(RefCell::new(Mutation {
                    name,
                    stage: stage.to_owned(),
                    log: Rc::clone(&log),
                    tokens: Rc::downgrade(&tokens),
                })),
            );
            tokens.borrow_mut().push(token);
        }
        let result = parser.start(71).and_then(|()| match stage {
            "start" | "shift" => Ok(()),
            "put" => parser.put(&[65]),
            "end" => parser.end(true, true).map(|_| ()),
            "reset" => parser.reset(),
            _ => panic!("unknown fixture phase {stage}"),
        });
        let expected = if stage == "shift" {
            Ok(())
        } else {
            Err(DispatchError::ActiveHandlerRemoved)
        };
        assert_eq!(result, expected, "{stage}");
        assert_eq!(log.borrow().join("|"), fields[1], "{stage}");
        if stage == "shift" {
            assert_eq!(fields[2], "none");
            continue;
        }
        assert_eq!(fields[2], "TypeError");
        // A thrown upstream callback traversal leaves the current identifier in
        // place. The error must not report success and reset state behind callers.
        let output = Rc::new(RefCell::new(Vec::new()));
        let capture = Rc::clone(&output);
        parser.set_handler_fallback(move |id, action| capture.borrow_mut().push((id, action)));
        parser.put(&[66]).unwrap();
        assert_eq!(*output.borrow(), [(71, ApcAction::Put(vec![66]))]);
    }
}

fn register(parser: &mut ApcParser, ident: u32, name: &'static str, log: &Log) -> DisposableHandle {
    let log = Rc::clone(log);
    parser.register_handler(
        ident,
        Rc::new(RefCell::new(ApcHandler::new(move |data: Vec<u16>| {
            log.borrow_mut().push(format!("{name}:{}", data.len()));
            HandlerResult::Ready(false)
        }))),
    )
}
#[test]
fn identifiers_are_separate_and_dropped_tokens_do_not_unregister() {
    let mut parser = ApcParser::new();
    let log = Log::default();
    drop(register(&mut parser, 71, "a", &log));
    drop(register(&mut parser, 72, "b", &log));
    parser.start(71).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    parser.start(72).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*log.borrow(), ["a:0", "b:0"]);
}
#[test]
fn duplicate_identity_disposal_removes_first_match_repeatedly() {
    let mut parser = ApcParser::new();
    let count = Rc::new(RefCell::new(0));
    let output = Rc::clone(&count);
    let handler = Rc::new(RefCell::new(ApcHandler::new(move |_| {
        *output.borrow_mut() += 1;
        HandlerResult::Ready(false)
    })));
    let token = parser.register_handler(71, handler.clone());
    parser.register_handler(71, handler);
    parser.start(71).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*count.borrow(), 2);
    token.dispose();
    parser.start(71).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*count.borrow(), 3);
    token.dispose();
    token.dispose();
    parser.start(71).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*count.borrow(), 3);
}
#[test]
fn cleared_active_list_is_distinct_from_new_registration_list() {
    let mut parser = ApcParser::new();
    let log = Log::default();
    register(&mut parser, 71, "old", &log);
    parser.start(71).unwrap();
    parser.clear_handler(71);
    register(&mut parser, 71, "new", &log);
    parser.put(&[65]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    parser.start(71).unwrap();
    parser.put(&[66, 67]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*log.borrow(), ["old:1", "new:2"]);
}
#[test]
fn registration_during_active_payload_joins_the_shared_list() {
    let mut parser = ApcParser::new();
    let log = Log::default();
    register(&mut parser, 71, "old", &log);
    parser.start(71).unwrap();
    parser.put(&[65]).unwrap();
    register(&mut parser, 71, "new", &log);
    parser.put(&[66]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*log.borrow(), ["new:1", "old:2"]);
}
#[test]
fn dispose_does_not_end_active_handlers_and_allows_new_registration() {
    let mut parser = ApcParser::new();
    let log = Log::default();
    let old = register(&mut parser, 71, "old", &log);
    parser.start(71).unwrap();
    parser.dispose();
    old.dispose();
    assert!(log.borrow().is_empty());
    register(&mut parser, 71, "new", &log);
    parser.start(71).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    assert_eq!(*log.borrow(), ["new:0"]);
}
