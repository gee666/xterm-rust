// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::parser::{
    apc::ApcHandler,
    dcs::DcsHandler,
    escape::{EscapeSequenceParser, FunctionIdentifier},
    handler::HandlerResult,
    osc::OscHandler,
    params::Params,
};
fn data(text: &str) -> Vec<u32> {
    text.chars().map(u32::from).collect()
}
#[test]
fn disposal_during_execute_clears_maps_but_keeps_print_and_fallbacks() {
    let mut p = EscapeSequenceParser::new();
    let log = Rc::new(RefCell::new(Vec::new()));
    let out = Rc::clone(&log);
    p.set_print_handler(move |text| {
        out.borrow_mut().push(format!(
            "p:{}",
            text.iter()
                .copied()
                .map(|c| char::from_u32(c).unwrap())
                .collect::<String>()
        ))
    });
    let dispose = p.disposal_handle();
    let out = Rc::clone(&log);
    p.set_execute_handler(7, move |_| {
        out.borrow_mut().push("bell".to_owned());
        dispose.dispose();
        true
    });
    let out = Rc::clone(&log);
    p.set_execute_handler(10, move |_| {
        out.borrow_mut().push("LF".to_owned());
        true
    });
    let out = Rc::clone(&log);
    p.set_execute_handler_fallback(move |code| out.borrow_mut().push(format!("ef:{code}")));
    let out = Rc::clone(&log);
    p.set_csi_handler_fallback(move |id, _| out.borrow_mut().push(format!("cf:{id}")));
    let out = Rc::clone(&log);
    p.set_esc_handler_fallback(move |id| out.borrow_mut().push(format!("sf:{id}")));
    let out = Rc::clone(&log);
    p.register_csi_handler(
        FunctionIdentifier::new("m"),
        Rc::new(RefCell::new(move |_: &Params| {
            out.borrow_mut().push("CSI".to_owned());
            HandlerResult::Ready(true)
        })),
    )
    .unwrap();
    let out = Rc::clone(&log);
    p.register_esc_handler(
        FunctionIdentifier::new("c"),
        Rc::new(RefCell::new(move |_: &Params| {
            out.borrow_mut().push("RIS".to_owned());
            HandlerResult::Ready(true)
        })),
    )
    .unwrap();
    assert!(p
        .parse(&data("A\x07B\n\x1b[mC\x1bcD"), None)
        .unwrap()
        .is_none());
    assert_eq!(
        *log.borrow(),
        ["p:A", "bell", "p:B", "ef:10", "cf:109", "p:C", "sf:99", "p:D"]
    );
}
#[test]
fn disposal_during_callback_immediately_clears_all_string_children() {
    let mut p = EscapeSequenceParser::new();
    let calls = Rc::new(Cell::new(0));
    let out = Rc::clone(&calls);
    p.osc.register_handler(
        1.0,
        Rc::new(RefCell::new(OscHandler::new(move |_| {
            out.set(out.get() + 1);
            HandlerResult::Ready(true)
        }))),
    );
    let out = Rc::clone(&calls);
    p.register_dcs_handler(
        FunctionIdentifier::new("q"),
        Rc::new(RefCell::new(DcsHandler::new(move |_, _| {
            out.set(out.get() + 1);
            HandlerResult::Ready(true)
        }))),
    )
    .unwrap();
    let out = Rc::clone(&calls);
    p.register_apc_handler(
        FunctionIdentifier::new("X"),
        Rc::new(RefCell::new(ApcHandler::new(move |_| {
            out.set(out.get() + 1);
            HandlerResult::Ready(true)
        }))),
    )
    .unwrap();
    let handle = p.disposal_handle();
    p.set_execute_handler(7, move |_| {
        handle.dispose();
        true
    });
    assert!(p
        .parse(
            &data("\x07\x1b]1;abc\x07\x1bPqabc\x1b\\\x1b_Xabc\x1b\\"),
            None
        )
        .unwrap()
        .is_none());
    assert_eq!(calls.get(), 0);
}
#[test]
fn repeated_parent_dispose_does_not_remove_new_registrations() {
    let mut p = EscapeSequenceParser::new();
    let handle = p.disposal_handle();
    handle.dispose();
    let calls = Rc::new(Cell::new(0));
    let out = Rc::clone(&calls);
    p.set_execute_handler(10, move |_| {
        out.set(out.get() + 1);
        true
    });
    let out = Rc::clone(&calls);
    p.register_csi_handler(
        FunctionIdentifier::new("m"),
        Rc::new(RefCell::new(move |_: &Params| {
            out.set(out.get() + 1);
            HandlerResult::Ready(true)
        })),
    )
    .unwrap();
    p.dispose();
    handle.dispose();
    assert!(p.parse(&data("\n\x1b[m"), None).unwrap().is_none());
    assert_eq!(calls.get(), 2);
    assert!(handle.is_disposed());
}
#[test]
fn disposal_handles_do_not_keep_parser_callbacks_alive() {
    let mut p = EscapeSequenceParser::new();
    let handle = p.disposal_handle();
    let sentinel = Rc::new(());
    let weak = Rc::downgrade(&sentinel);
    let captured = handle.clone();
    p.set_execute_handler(7, move |_| {
        let _owner = &sentinel;
        captured.dispose();
        true
    });
    drop(p);
    assert!(weak.upgrade().is_none());
    handle.dispose();
}
