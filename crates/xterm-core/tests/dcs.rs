// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Assertions ported from DcsParser.test.ts.

#[path = "dcs_cases/callback.rs"]
mod callback;
mod dcs_support;
#[path = "dcs_cases/mutation.rs"]
mod mutation;
#[path = "dcs_cases/oracle.rs"]
mod oracle;
#[path = "dcs_cases/raw_errors.rs"]
mod raw_errors;

use dcs_support::*;
use std::rc::Rc;

#[test]
fn set_dcs_handler() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "th", true, false));
    payload(&mut parser);
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), expected(&["th"], true));
}

#[test]
fn clear_dcs_handler() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "th", true, false));
    parser.clear_handler(IDENT);
    payload(&mut parser);
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), expected(&[&IDENT.to_string()], true));
}

#[test]
fn add_dcs_handler() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "th1", true, false));
    parser.register_handler(IDENT, handler(&reports, "th2", true, false));
    payload(&mut parser);
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), expected(&["th2", "th1"], true));
}

#[test]
fn add_dcs_handler_with_return_false() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "th1", true, false));
    parser.register_handler(IDENT, handler(&reports, "th2", false, false));
    payload(&mut parser);
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), expected(&["th2", "th1"], false));
}

#[test]
fn dispose_handlers() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "th1", true, false));
    let disposable = parser.register_handler(IDENT, handler(&reports, "th2", false, false));
    disposable.dispose();
    payload(&mut parser);
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), expected(&["th1"], true));
}

#[test]
fn sync_async_sync_first_handles_then_cleanup() {
    mixed(&[false, true, false], &["s1", "a1", "s2"], true);
}

#[test]
fn sync_async_sync_all_run() {
    mixed(&[false, true, false], &["s1", "a1", "s2"], false);
}

#[test]
fn async_sync_async_first_handles_then_cleanup() {
    mixed(&[true, false, true], &["a1", "s1", "a2"], true);
}

#[test]
fn async_sync_async_all_run() {
    mixed(&[true, false, true], &["a1", "s1", "a2"], false);
}

fn mixed(asynchronous: &[bool], labels: &[&str], handled: bool) {
    let (mut parser, reports) = parser();
    for (&asynchronous, &label) in asynchronous.iter().zip(labels) {
        parser.register_handler(IDENT, handler(&reports, label, handled, asynchronous));
    }
    payload(&mut parser);
    unhook(&mut parser, true);
    let labels: Vec<_> = labels.iter().copied().rev().collect();
    assert_eq!(*reports.borrow(), expected(&labels, handled));
}

#[test]
fn reset_aborts_active_handlers_then_parser_can_be_reused() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "th", true, false));
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    parser.put(&input("partial")).unwrap();
    parser.reset().unwrap();
    assert_eq!(
        *reports.borrow(),
        vec![
            "th:H:1,2,3".to_owned(),
            format!(
                "th:P:{}",
                units(&"partial".encode_utf16().collect::<Vec<_>>())
            ),
            "th:U:false".to_owned(),
        ]
    );
    reports.borrow_mut().clear();
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    parser.put(&input("complete")).unwrap();
    unhook(&mut parser, true);
    assert_eq!(
        *reports.borrow(),
        vec![
            "th:H:1,2,3".to_owned(),
            format!(
                "th:P:{}",
                units(&"complete".encode_utf16().collect::<Vec<_>>())
            ),
            "th:U:true".to_owned(),
        ]
    );
}

#[test]
fn clear_during_payload_keeps_original_active_list() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "th", true, false));
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    parser.clear_handler(IDENT);
    parser.put(&input("Here comes")).unwrap();
    parser.put(&input("the mouse!")).unwrap();
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), expected(&["th"], true));
}

#[test]
fn duplicate_identity_disposal_removes_first_match_each_time() {
    let (mut parser, reports) = parser();
    let shared = handler(&reports, "th", false, false);
    let disposable = parser.register_handler(IDENT, Rc::clone(&shared));
    parser.register_handler(IDENT, shared);
    disposable.dispose();
    payload(&mut parser);
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), expected(&["th"], true));
    reports.borrow_mut().clear();
    disposable.dispose();
    payload(&mut parser);
    unhook(&mut parser, true);
    assert_eq!(*reports.borrow(), expected(&[&IDENT.to_string()], true));
}

#[test]
fn reset_retains_async_cleanup_for_the_executor() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "pending", false, true));
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    parser.reset().unwrap();
    parser.dispose();
    assert_eq!(*reports.borrow(), ["pending:H:1,2,3"]);
    let cleanup = parser.take_cleanup();
    assert_eq!(cleanup.len(), 1);
    for result in cleanup {
        assert!(!resolve(result));
    }
    assert_eq!(*reports.borrow(), ["pending:H:1,2,3", "pending:U:false"]);
    assert!(parser.take_cleanup().is_empty());
}

#[test]
fn pending_cleanup_result_never_restarts_bubbling() {
    for handled in [false, true] {
        let (mut parser, reports) = parser();
        parser.register_handler(IDENT, handler(&reports, "old", true, false));
        parser.register_handler(IDENT, handler(&reports, "pending", handled, true));
        parser.register_handler(IDENT, handler(&reports, "new", true, false));
        payload(&mut parser);
        unhook(&mut parser, true);
        assert_eq!(
            *reports.borrow(),
            expected(&["new", "pending", "old"], true)
        );
    }
}

#[test]
fn implicit_hook_reset_enqueues_cleanup_without_losing_new_sequence() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "pending", false, true));
    parser.hook(IDENT, &params(&[1])).unwrap();
    parser.hook(IDENT, &params(&[2])).unwrap();
    for result in parser.take_cleanup() {
        assert!(!resolve(result));
    }
    unhook(&mut parser, true);
    assert_eq!(
        *reports.borrow(),
        [
            "pending:H:1",
            "pending:H:2",
            "pending:U:false",
            "pending:U:true"
        ]
    );
}

#[test]
fn reset_while_paused_skips_already_unhooked_handlers() {
    let (mut parser, reports) = parser();
    parser.register_handler(IDENT, handler(&reports, "first", true, false));
    parser.register_handler(IDENT, handler(&reports, "pending", false, true));
    parser.hook(IDENT, &params(&[1, 2, 3])).unwrap();
    let result = parser.unhook(true, true).unwrap().unwrap();
    assert!(!resolve(result));
    parser.reset().unwrap();
    assert_eq!(
        *reports.borrow(),
        [
            "pending:H:1,2,3",
            "first:H:1,2,3",
            "pending:U:true",
            "first:U:false"
        ]
    );
}
