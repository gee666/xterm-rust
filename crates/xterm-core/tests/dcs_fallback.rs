// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under MIT. Recoverable equivalents of source fallback exceptions.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{
    event::EmitError,
    parser::{
        dcs::{DcsAction, DcsHandler, DcsParser},
        handler::{DispatchError, HandlerResult},
        params::{ParamValue, Params},
    },
};
const IDENT: i32 = 0x2b70;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Hook,
    Put,
    Unhook,
}
fn params() -> Params {
    Params::from_array(&[ParamValue::Param(1)]).unwrap()
}

#[test]
fn fallback_errors_retain_exact_cause_and_ident_until_successful_unhook() {
    for phase in [Phase::Hook, Phase::Put, Phase::Unhook] {
        let mut parser = DcsParser::new();
        let cause = EmitError::callback(std::io::Error::other("exact diagnostic failure"));
        let expected = cause.clone();
        let fail = Cell::new(true);
        let reports = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&reports);
        parser.set_handler_fallback_checked(move |ident, action| {
            let current = match action {
                DcsAction::Hook(_) => Phase::Hook,
                DcsAction::Put(_) => Phase::Put,
                DcsAction::Unhook(_) => Phase::Unhook,
            };
            output.borrow_mut().push(ident);
            if current == phase && fail.replace(false) {
                Err(DispatchError::Callback(cause.clone()))
            } else {
                Ok(())
            }
        });
        let hook = parser.hook(IDENT, &params());
        let error = match phase {
            Phase::Hook => {
                let error = hook.unwrap_err();
                parser.put(&[65]).unwrap();
                error
            }
            Phase::Put => {
                hook.unwrap();
                let error = parser.put(&[65]).unwrap_err();
                parser.put(&[66]).unwrap();
                error
            }
            Phase::Unhook => {
                hook.unwrap();
                parser.put(&[65]).unwrap();
                let error = parser.unhook(true, true).err().unwrap();
                parser.put(&[66]).unwrap();
                error
            }
        };
        let DispatchError::Callback(actual) = error else {
            panic!("typed callback cause was discarded")
        };
        assert_eq!(actual, expected);
        assert!(std::ptr::eq(
            actual.callback_ref::<std::io::Error>().unwrap(),
            expected.callback_ref::<std::io::Error>().unwrap()
        ));
        assert!(parser.unhook(true, true).unwrap().is_none());
        parser.put(&[]).unwrap();
        let reports = reports.borrow();
        assert_eq!(reports.last(), Some(&0));
        assert!(reports[..reports.len() - 1]
            .iter()
            .all(|ident| *ident == IDENT));
    }
}

#[test]
fn registered_handlers_never_fall_back_even_when_every_handler_declines() {
    let mut parser = DcsParser::new();
    let called = Rc::new(Cell::new(0));
    let calls = Rc::clone(&called);
    parser.set_handler_fallback_checked(move |_, _| {
        calls.set(calls.get() + 1);
        Err(DispatchError::CallbackFailed)
    });
    parser.register_handler(
        IDENT,
        Rc::new(RefCell::new(DcsHandler::new(|_, _| {
            HandlerResult::Ready(false)
        }))),
    );
    parser.hook(IDENT, &params()).unwrap();
    parser.put(&[65]).unwrap();
    assert!(parser.unhook(true, true).unwrap().is_none());
    assert_eq!(called.get(), 0);
}

#[test]
fn disposal_removes_a_checked_fallback_and_allows_source_style_reuse() {
    let mut parser = DcsParser::new();
    parser.set_handler_fallback_checked(|_, _| Err(DispatchError::CallbackFailed));
    parser.dispose();
    parser.hook(IDENT, &params()).unwrap();
    parser.put(&[65]).unwrap();
    assert!(parser.unhook(true, true).unwrap().is_none());
}
