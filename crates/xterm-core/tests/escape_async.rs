// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Faithful assertions from EscapeSequenceParser.test.ts async.
mod escape_async_support;
use escape_async_support::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::parser::{
    escape::{EscapeSequenceParser, FunctionIdentifier, HandlerKind, ParseError, ParseStackState},
    handler::{DispatchError, HandlerResult},
    params::Params,
};

#[test]
fn sync_handlers_keep_sync_mode_and_produce_original_result() {
    let (mut parser, log) = setup(false);
    assert!(parser.parse(&data(INPUT), None).unwrap().is_none());
    assert_eq!(parser.parse_stack_state(), ParseStackState::None);
    assert_eq!(*log.borrow(), EXPECTED);
    log.borrow_mut().clear();
    assert!(drive(&mut parser, &data(INPUT)).is_empty());
    assert_eq!(*log.borrow(), EXPECTED);
}

#[test]
fn async_handlers_require_continuation_and_remain_failed_until_reset() {
    let (mut parser, log) = setup(true);
    assert!(parser.parse(&data(INPUT), None).unwrap().is_some());
    assert_ne!(*log.borrow(), EXPECTED);
    assert_eq!(parser.parse_stack_state(), ParseStackState::Paused);
    assert!(matches!(
        parser.parse(&data(INPUT), None),
        Err(ParseError::ImproperContinuation)
    ));
    assert_eq!(parser.parse_stack_state(), ParseStackState::Failed);
    for previous in [None, Some(true), Some(false)] {
        assert!(matches!(
            parser.parse(&data("foobar"), previous),
            Err(ParseError::ImproperContinuation)
        ));
    }
    parser.reset().unwrap();
    assert_eq!(parser.parse_stack_state(), ParseStackState::Reset);
    // Upstream reset skips the saved codepoint, even if a different chunk is
    // supplied. Normal terminal adapters must always keep the original chunk.
    drive(&mut parser, &data(INPUT));
    assert_eq!(*log.borrow(), EXPECTED);
    assert_eq!(parser.parse_stack_state(), ParseStackState::None);
}

#[test]
fn reset_during_async_pause_continues_at_next_codepoint_in_same_chunk() {
    let (mut parser, log) = setup(true);
    let chunk = data("\x1b[1mXY");
    let pending = parser.parse(&chunk, None).unwrap().unwrap();
    assert_eq!(parser.pause_info().unwrap().chunk_position, 3);
    parser.reset().unwrap();
    assert_eq!(parser.parse_stack_state(), ParseStackState::Reset);
    assert!(parser
        .parse(&chunk, Some(resolve(pending)))
        .unwrap()
        .is_none());
    assert_eq!(*log.borrow(), ["SGR:[1]", "PRINT:XY"]);
    assert_eq!(parser.parse_stack_state(), ParseStackState::None);
}

#[test]
fn awaited_parse_preserves_original_results_and_exact_stack_positions() {
    let (mut parser, log) = setup(true);
    let pauses = drive(&mut parser, &data(INPUT));
    assert_eq!(*log.borrow(), EXPECTED);
    use HandlerKind::*;
    assert_eq!(
        pauses
            .iter()
            .map(|p| (p.chunk_position, p.kind, p.handler_position))
            .collect::<Vec<_>>(),
        [
            (6, Csi, 0),
            (15, Esc, 0),
            (20, Esc, 0),
            (27, Csi, 0),
            (41, Dcs, 0),
            (54, Osc, 0),
            (62, Apc, 0)
        ]
    );
    assert_eq!(parser.parse_stack_state(), ParseStackState::None);
}

#[test]
fn one_codepoint_chunks_preserve_original_callback_splitting() {
    let (mut parser, log) = setup(true);
    let mut pauses = Vec::new();
    for code in data(INPUT) {
        pauses.extend(drive(&mut parser, &[code]));
    }
    assert_eq!(
        *log.borrow(),
        [
            "SGR:[1, 31]",
            "PRINT:h",
            "PRINT:e",
            "PRINT:l",
            "PRINT:l",
            "PRINT:o",
            "PRINT: ",
            "ESC:%G",
            "PRINT:w",
            "PRINT:o",
            "PRINT:r",
            "ESC:E",
            "PRINT:l",
            "PRINT:d",
            "PRINT:!",
            "SGR:[0]",
            "EXE:\r",
            "EXE:\n",
            "PRINT:$",
            "PRINT:>",
            "DCS:xyz:[1, 2]",
            "OSC:foo=bar",
            "APC:abc",
            "PRINT:F",
            "PRINT:I",
            "PRINT:N"
        ]
    );
    assert_eq!(pauses.len(), 7);
    assert!(pauses
        .iter()
        .all(|pause| pause.chunk_position == 0 && pause.handler_position == 0));
}

#[test]
fn multiple_async_handlers_bubble_suppress_and_dispose_in_all_five_families() {
    use HandlerKind::*;
    for (kind, original) in [
        (Csi, "SGR:"),
        (Esc, "ESC:E"),
        (Dcs, "DCS:"),
        (Osc, "OSC:"),
        (Apc, "APC:"),
    ] {
        let (mut parser, log) = setup(true);
        let token = extra(&mut parser, kind, &log, "extra", true, false);
        let pauses = drive(&mut parser, &data(INPUT));
        let count = if kind == Csi { 2 } else { 1 };
        assert_eq!(
            log.borrow()
                .iter()
                .filter(|entry| entry.as_str() == "extra")
                .count(),
            count
        );
        for (index, entry) in log.borrow().iter().enumerate() {
            if entry == "extra" {
                assert!(log.borrow()[index + 1].starts_with(original));
            }
        }
        let original_pauses = [
            (6, Csi, 0),
            (15, Esc, 0),
            (20, Esc, 0),
            (27, Csi, 0),
            (41, Dcs, 0),
            (54, Osc, 0),
            (62, Apc, 0),
        ];
        let mut expected = Vec::new();
        for entry in original_pauses {
            if entry.1 == kind && !(kind == Esc && entry.0 == 15) {
                expected.push((entry.0, kind, usize::from(matches!(kind, Csi | Esc))));
            }
            expected.push(entry);
        }
        assert_eq!(
            pauses
                .iter()
                .map(|p| (p.chunk_position, p.kind, p.handler_position))
                .collect::<Vec<_>>(),
            expected
        );
        token.dispose();
        token.dispose();
        log.borrow_mut().clear();
        assert_eq!(drive(&mut parser, &data(INPUT)).len(), 7);
        assert_eq!(*log.borrow(), EXPECTED);
        let token = extra(&mut parser, kind, &log, "handled", true, true);
        log.borrow_mut().clear();
        let pauses = drive(&mut parser, &data(INPUT));
        let expected: Vec<_> = original_pauses
            .into_iter()
            .map(|(position, family, _)| {
                (
                    position,
                    family,
                    usize::from(family == kind && matches!(kind, Csi | Esc) && position != 15),
                )
            })
            .collect();
        assert_eq!(
            pauses
                .iter()
                .map(|p| (p.chunk_position, p.kind, p.handler_position))
                .collect::<Vec<_>>(),
            expected
        );
        assert!(!log.borrow().iter().any(|entry| entry.starts_with(original)));
        token.dispose();
        log.borrow_mut().clear();
        drive(&mut parser, &data(INPUT));
        assert_eq!(*log.borrow(), EXPECTED);
    }
}

#[test]
fn mixed_sync_async_sgr_handlers_keep_original_order_and_stack_positions() {
    let (mut parser, log) = setup(true);
    let sync = extra(&mut parser, HandlerKind::Csi, &log, "sync", false, false);
    let asynchronous = extra(&mut parser, HandlerKind::Csi, &log, "async", true, false);
    let pauses = drive(&mut parser, &data(INPUT));
    for (index, entry) in log.borrow().iter().enumerate() {
        if entry == "async" {
            assert_eq!(log.borrow()[index + 1], "sync");
            assert!(log.borrow()[index + 2].starts_with("SGR:"));
        }
    }
    assert_eq!(
        pauses
            .iter()
            .filter(|p| p.kind == HandlerKind::Csi)
            .map(|p| (p.chunk_position, p.handler_position))
            .collect::<Vec<_>>(),
        [(6, 2), (6, 0), (27, 2), (27, 0)]
    );
    sync.dispose();
    log.borrow_mut().clear();
    let pauses = drive(&mut parser, &data(INPUT));
    assert_eq!(
        pauses
            .iter()
            .filter(|p| p.kind == HandlerKind::Csi)
            .map(|p| p.handler_position)
            .collect::<Vec<_>>(),
        [1, 0, 1, 0]
    );
    asynchronous.dispose();
    log.borrow_mut().clear();
    drive(&mut parser, &data(INPUT));
    assert_eq!(*log.borrow(), EXPECTED);
}

#[test]
fn asynchronous_false_exhaustion_does_not_call_fallback_matching_source() {
    for csi in [true, false] {
        let mut parser = EscapeSequenceParser::new();
        let called = Rc::new(Cell::new(false));
        let target = Rc::clone(&called);
        parser.set_csi_handler_fallback(move |_, _| target.set(true));
        let target = Rc::clone(&called);
        parser.set_esc_handler_fallback(move |_| target.set(true));
        let handler = Rc::new(RefCell::new(|_: &Params| result(true, false)));
        if csi {
            parser
                .register_csi_handler(FunctionIdentifier::new("m"), handler)
                .unwrap();
        } else {
            parser
                .register_esc_handler(FunctionIdentifier::new("E"), handler)
                .unwrap();
        }
        drive(&mut parser, &data(if csi { "\x1b[m" } else { "\x1bE" }));
        assert!(!called.get());
    }
}

#[test]
fn callback_removal_during_reverse_traversal_is_not_silent_success() {
    let mut parser = EscapeSequenceParser::new();
    let tokens = Rc::new(RefCell::new(Vec::new()));
    let handler = Rc::new(RefCell::new(|_: &Params| HandlerResult::Ready(false)));
    tokens.borrow_mut().push(
        parser
            .register_csi_handler(FunctionIdentifier::new("m"), handler)
            .unwrap(),
    );
    let remove = Rc::clone(&tokens);
    tokens.borrow_mut().push(
        parser
            .register_csi_handler(
                FunctionIdentifier::new("m"),
                Rc::new(RefCell::new(move |_: &Params| {
                    for token in remove.borrow().iter() {
                        token.dispose();
                    }
                    HandlerResult::Ready(false)
                })),
            )
            .unwrap(),
    );
    assert!(matches!(
        parser.parse(&data("\x1b[m"), None),
        Err(ParseError::Dispatch(DispatchError::ActiveHandlerRemoved))
    ));
}
