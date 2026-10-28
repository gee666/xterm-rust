// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. DcsHandlerFactory upstream assertions.

use crate::dcs_support::*;
use std::cell::RefCell;
use std::rc::Rc;
use xterm_core::parser::dcs::{DcsHandler, DcsHandlerTrait, HandlerHandle};
use xterm_core::parser::handler::{DispatchError, HandlerResult};
use xterm_core::parser::params::{ParamValue, Params};

fn buffered(reports: &Reports, label: &str, handled: bool, asynchronous: bool) -> HandlerHandle {
    let reports = Rc::clone(reports);
    let label = label.to_owned();
    Rc::new(RefCell::new(DcsHandler::with_limit(
        100,
        move |data, params| {
            let report = format!("{label}:{}:{}", params_text(params), units(data));
            if asynchronous {
                let reports = Rc::clone(&reports);
                HandlerResult::Pending(Box::pin(async move {
                    reports.borrow_mut().push(report);
                    handled
                }))
            } else {
                reports.borrow_mut().push(report);
                HandlerResult::Ready(handled)
            }
        },
    )))
}

fn payload_with_space(parser: &mut xterm_core::parser::dcs::DcsParser) {
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    parser.put(&input("Here comes")).unwrap();
    parser.put(&input(" the mouse!")).unwrap();
}

fn report(label: &str, text: &str) -> String {
    format!(
        "{label}:1,2,3:{}",
        units(&text.encode_utf16().collect::<Vec<_>>())
    )
}

#[test]
fn called_once_on_success_sync_and_async() {
    for asynchronous in [false, true] {
        let (mut parser, reports) = parser();
        parser.register_handler(IDENT, buffered(&reports, "th", true, asynchronous));
        payload_with_space(&mut parser);
        unhook(&mut parser, true);
        assert_eq!(*reports.borrow(), [report("th", "Here comes the mouse!")]);
    }
}

#[test]
fn not_called_on_abort_sync_and_async() {
    for asynchronous in [false, true] {
        let (mut parser, reports) = parser();
        parser.register_handler(IDENT, buffered(&reports, "th", true, asynchronous));
        payload_with_space(&mut parser);
        unhook(&mut parser, false);
        assert!(reports.borrow().is_empty());
    }
}

#[test]
fn disposable_sync_and_async() {
    for asynchronous in [false, true] {
        let (mut parser, reports) = parser();
        parser.register_handler(IDENT, buffered(&reports, "one", true, asynchronous));
        let disposable =
            parser.register_handler(IDENT, buffered(&reports, "two", true, asynchronous));
        payload_with_space(&mut parser);
        unhook(&mut parser, true);
        assert_eq!(*reports.borrow(), [report("two", "Here comes the mouse!")]);
        disposable.dispose();
        parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
        parser.put(&input("some other")).unwrap();
        parser.put(&input(" data")).unwrap();
        unhook(&mut parser, true);
        assert_eq!(
            *reports.borrow(),
            [
                report("two", "Here comes the mouse!"),
                report("one", "some other data")
            ]
        );
    }
}

#[test]
fn respects_return_false_sync_and_async() {
    for asynchronous in [false, true] {
        let (mut parser, reports) = parser();
        parser.register_handler(IDENT, buffered(&reports, "one", true, asynchronous));
        parser.register_handler(IDENT, buffered(&reports, "two", false, asynchronous));
        payload_with_space(&mut parser);
        unhook(&mut parser, true);
        assert_eq!(
            *reports.borrow(),
            [
                report("two", "Here comes the mouse!"),
                report("one", "Here comes the mouse!")
            ]
        );
    }
}

#[test]
fn works_up_to_payload_limit() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, buffered(&reports, "th", true, false));
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    for _ in 0..10 {
        parser.put(&input("AAAAAAAAAA")).unwrap();
    }
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), [report("th", &"A".repeat(100))]);
}

#[test]
fn aborts_at_payload_limit_plus_one() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, buffered(&reports, "th", true, false));
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    for _ in 0..10 {
        parser.put(&input("AAAAAAAAAA")).unwrap();
    }
    parser.put(&input("A")).unwrap();
    unhook(&mut parser, true);
    assert!(reports.borrow().is_empty());
}

#[test]
fn utf16_limit_and_lone_surrogates_are_preserved() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let mut handler = DcsHandler::with_limit(3, move |data, params| {
        output.borrow_mut().push((data.to_vec(), params.to_array()));
        HandlerResult::Ready(true)
    });
    handler.hook(&params(&[0]));
    handler.put(&[0x10000, 0xd800]);
    assert!(resolve(handler.unhook(true)));
    assert_eq!(
        *calls.borrow(),
        [(vec![0xd800, 0xdc00, 0xd800], vec![ParamValue::Param(0)])]
    );
    handler.hook(&params(&[0]));
    handler.put(&[0x10000, 0x10000]);
    assert!(!resolve(handler.unhook(true)));
    assert_eq!(calls.borrow().len(), 1);
}

#[test]
fn overflow_latch_clears_on_hook_and_empty_payload_is_valid() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let mut handler = DcsHandler::with_limit(1, move |data, params| {
        output.borrow_mut().push((data.to_vec(), params.to_array()));
        HandlerResult::Ready(true)
    });
    handler.hook(&params(&[1]));
    handler.put(&[65, 66]);
    handler.put(&[67]);
    assert!(!resolve(handler.unhook(true)));
    assert!(calls.borrow().is_empty());
    handler.hook(&Params::default());
    handler.put(&[]);
    assert!(resolve(handler.unhook(true)));
    assert_eq!(
        *calls.borrow(),
        [(Vec::<u16>::new(), vec![ParamValue::Param(0)])]
    );
}

#[test]
fn async_resolution_clears_retained_data_and_params() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let mut handler = DcsHandler::new(move |data, params| {
        output.borrow_mut().push((data.to_vec(), params.to_array()));
        HandlerResult::Pending(Box::pin(async { true }))
    });
    handler.hook(&params(&[1]));
    handler.put(&[65]);
    let pending = handler.unhook(true);
    handler.put(&[66]);
    assert!(resolve(pending));
    assert!(resolve(handler.unhook(true)));
    assert_eq!(
        *calls.borrow(),
        [
            (vec![65], vec![ParamValue::Param(1)]),
            (vec![], vec![ParamValue::Param(0)]),
        ]
    );
}

#[test]
fn failed_callback_retains_payload_and_params_until_retry() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let mut handler = DcsHandler::new(move |data, params| {
        let mut calls = output.borrow_mut();
        calls.push((data.to_vec(), params.to_array()));
        if calls.len() == 1 {
            HandlerResult::Failed(DispatchError::CallbackFailed)
        } else {
            HandlerResult::Ready(true)
        }
    });
    handler.hook(&params(&[1]));
    handler.put(&[65]);
    assert!(matches!(
        handler.unhook(true),
        HandlerResult::Failed(DispatchError::CallbackFailed)
    ));
    handler.put(&[66]);
    assert!(resolve(handler.unhook(true)));
    assert_eq!(
        *calls.borrow(),
        [
            (vec![65], vec![ParamValue::Param(1)]),
            (vec![65, 66], vec![ParamValue::Param(1)]),
        ]
    );
}

#[test]
fn failed_callback_stops_bubbling_and_reset_can_recover() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "older", true, false));
    parser.register_handler(
        IDENT,
        Rc::new(RefCell::new(DcsHandler::new(|_, _| {
            HandlerResult::Failed(DispatchError::CallbackFailed)
        }))),
    );
    parser.hook(IDENT, &params(&[1])).unwrap();
    assert_eq!(
        parser.unhook(true, true).err(),
        Some(DispatchError::CallbackFailed)
    );
    assert_eq!(*reports.borrow(), ["older:H:1"]);
    parser.reset().unwrap();
    assert_eq!(*reports.borrow(), ["older:H:1", "older:U:false"]);
}

#[test]
fn rejected_future_retains_data_until_successful_future_completion() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let mut handler = DcsHandler::new(move |data, params| {
        let mut calls = output.borrow_mut();
        calls.push((data.to_vec(), params.to_array()));
        let result = if calls.len() == 1 {
            Err(DispatchError::CallbackFailed)
        } else {
            Ok(true)
        };
        HandlerResult::PendingFallible(Box::pin(async move { result }))
    });
    handler.hook(&params(&[1]));
    handler.put(&[65]);
    assert_eq!(
        resolve_result(handler.unhook(true)),
        Err(DispatchError::CallbackFailed)
    );
    handler.put(&[66]);
    assert_eq!(resolve_result(handler.unhook(true)), Ok(true));
    assert_eq!(resolve_result(handler.unhook(true)), Ok(true));
    assert_eq!(
        *calls.borrow(),
        [
            (vec![65], vec![ParamValue::Param(1)]),
            (vec![65, 66], vec![ParamValue::Param(1)]),
            (vec![], vec![ParamValue::Param(0)]),
        ]
    );
}

#[test]
fn params_are_cloned_and_single_zero_is_canonicalized() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&calls);
    let mut handler = DcsHandler::new(move |_, params| {
        output.borrow_mut().push(params.to_array());
        HandlerResult::Ready(true)
    });
    let mut original = params(&[1, 2, 3]);
    handler.hook(&original);
    original.reset();
    resolve(handler.unhook(true));
    assert_eq!(calls.borrow()[0], params(&[1, 2, 3]).to_array());
    let zero = Params::from_array(&[ParamValue::Param(0), ParamValue::SubParams(vec![5])]).unwrap();
    handler.hook(&zero);
    resolve(handler.unhook(true));
    assert_eq!(calls.borrow()[1], vec![ParamValue::Param(0)]);
    let original = Params::from_array(&[
        ParamValue::Param(1),
        ParamValue::SubParams(vec![-1, 7]),
        ParamValue::Param(0),
    ])
    .unwrap();
    handler.hook(&original);
    resolve(handler.unhook(true));
    assert_eq!(calls.borrow()[2], original.to_array());
}
