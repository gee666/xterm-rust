// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Callback-safe registration and ownership regressions.
use std::cell::RefCell;
use std::rc::Rc;
use xterm_core::lifecycle::DisposableHandle;
use xterm_core::parser::handler::HandlerResult;
use xterm_core::parser::osc::{HandlerHandle, OscHandler, OscParser};

#[test]
fn callback_can_register_and_clear_without_reborrowing_parser() {
    for clear in [false, true] {
        let log = Rc::new(RefCell::new(Vec::new()));
        let mut parser = OscParser::new();
        let registrar = parser.registrar();
        let output = Rc::clone(&log);
        parser.register_handler(
            2.0,
            Rc::new(RefCell::new(OscHandler::new(move |data| {
                output.borrow_mut().push(("old", data));
                if clear {
                    registrar.clear_handler(2.0).unwrap();
                }
                let output = Rc::clone(&output);
                registrar
                    .register_handler(
                        2.0,
                        Rc::new(RefCell::new(OscHandler::new(move |data| {
                            output.borrow_mut().push(("new", data));
                            HandlerResult::Ready(true)
                        }))),
                    )
                    .unwrap();
                HandlerResult::Ready(true)
            }))),
        );
        parser.start().unwrap();
        parser.put(&[50, 59, 65]).unwrap();
        assert!(parser.end(true, true).unwrap().is_none());
        parser.start().unwrap();
        parser.put(&[50, 59, 66]).unwrap();
        assert!(parser.end(true, true).unwrap().is_none());
        assert_eq!(*log.borrow(), [("old", vec![65]), ("new", vec![66])]);
    }
}
#[test]
fn token_captured_by_its_handler_does_not_create_reference_cycle() {
    let mut parser = OscParser::new();
    let slot: Rc<RefCell<Option<DisposableHandle>>> = Rc::default();
    let captured = Rc::clone(&slot);
    let handler: HandlerHandle = Rc::new(RefCell::new(OscHandler::new(move |_| {
        if let Some(token) = captured.borrow().as_ref() {
            token.dispose();
        }
        HandlerResult::Ready(true)
    })));
    let weak = Rc::downgrade(&handler);
    *slot.borrow_mut() = Some(parser.register_handler(2.0, handler));
    drop(parser);
    assert!(weak.upgrade().is_none());
    slot.borrow().as_ref().unwrap().dispose();
}
#[test]
fn registrar_is_weak_but_explicit_dispose_does_not_permanently_close_live_parser() {
    let mut parser = OscParser::new();
    let registrar = parser.registrar();
    parser.dispose();
    assert!(registrar
        .register_handler(
            2.0,
            Rc::new(RefCell::new(OscHandler::new(|_| HandlerResult::Ready(
                true
            ))))
        )
        .is_some());
    parser.start().unwrap();
    parser.put(&[50, 59, 65]).unwrap();
    assert!(parser.end(true, true).unwrap().is_none());
    drop(parser);
    assert!(registrar.clear_handler(2.0).is_none());
    assert!(registrar
        .register_handler(
            2.0,
            Rc::new(RefCell::new(OscHandler::new(|_| HandlerResult::Ready(
                true
            ))))
        )
        .is_none());
}
