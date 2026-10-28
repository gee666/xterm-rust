// Copyright (c) 2018 The xterm.js authors. All rights reserved. Licensed under MIT.
use super::*;
use xterm_core::{
    lifecycle::DisposableHandle,
    parser::{dcs::DcsHandler, osc::OscHandler},
};

pub(super) const INPUT: &str = "\x1b[1;31mhello \x1b%Gwor\x1bEld!\x1b[0m\r\n$>\x1b]1;foo=bar\x1b\\";
#[derive(Clone, Copy, Debug)]
pub(super) enum Kind {
    Esc,
    Csi,
    Osc,
    Dcs,
    Apc,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Call(pub usize, pub Vec<V>, pub String);
pub(super) type Calls = Rc<RefCell<Vec<Call>>>;

pub(super) fn register(
    parser: &mut EscapeSequenceParser,
    kind: Kind,
    label: usize,
    answer: bool,
    calls: &Calls,
) -> DisposableHandle {
    let calls = Rc::clone(calls);
    match kind {
        Kind::Esc | Kind::Csi => {
            let callback = Rc::new(RefCell::new(move |params: &Params| {
                calls.borrow_mut().push(Call(
                    label,
                    if matches!(kind, Kind::Csi) {
                        params.to_array()
                    } else {
                        vec![]
                    },
                    String::new(),
                ));
                HandlerResult::Ready(answer)
            }));
            if matches!(kind, Kind::Esc) {
                parser
                    .register_esc_handler(
                        FunctionIdentifier {
                            prefix: None,
                            intermediates: Some("%"),
                            final_byte: "G",
                        },
                        callback,
                    )
                    .unwrap()
            } else {
                parser
                    .register_csi_handler(FunctionIdentifier::new("m"), callback)
                    .unwrap()
            }
        }
        Kind::Osc | Kind::Apc => {
            let handler = Rc::new(RefCell::new(OscHandler::new(move |payload: Vec<u16>| {
                calls.borrow_mut().push(Call(label, vec![], text(&payload)));
                HandlerResult::Ready(answer)
            })));
            if matches!(kind, Kind::Osc) {
                parser.osc.register_handler(1.0, handler)
            } else {
                parser
                    .register_apc_handler(
                        FunctionIdentifier {
                            prefix: None,
                            intermediates: Some("+"),
                            final_byte: "p",
                        },
                        handler,
                    )
                    .unwrap()
            }
        }
        Kind::Dcs => parser
            .register_dcs_handler(
                FunctionIdentifier {
                    prefix: None,
                    intermediates: Some("+"),
                    final_byte: "p",
                },
                Rc::new(RefCell::new(DcsHandler::new(move |payload, params| {
                    calls
                        .borrow_mut()
                        .push(Call(label, params.to_array(), text(payload)));
                    HandlerResult::Ready(answer)
                }))),
            )
            .unwrap(),
    }
}
fn input(kind: Kind) -> &'static str {
    match kind {
        Kind::Dcs => "\x1bP1;2;3+pabc\x1b\\",
        Kind::Apc => "\x1b_+pabc\x1b\\",
        _ => INPUT,
    }
}
pub(super) fn clear(parser: &mut EscapeSequenceParser, kind: Kind) {
    match kind {
        Kind::Esc => parser
            .clear_esc_handler(FunctionIdentifier {
                prefix: None,
                intermediates: Some("%"),
                final_byte: "G",
            })
            .unwrap(),
        Kind::Csi => parser
            .clear_csi_handler(FunctionIdentifier::new("m"))
            .unwrap(),
        Kind::Osc => parser.osc.clear_handler(1.0),
        Kind::Dcs => parser
            .clear_dcs_handler(FunctionIdentifier {
                prefix: None,
                intermediates: Some("+"),
                final_byte: "p",
            })
            .unwrap(),
        Kind::Apc => parser
            .clear_apc_handler(FunctionIdentifier {
                prefix: None,
                intermediates: Some("+"),
                final_byte: "p",
            })
            .unwrap(),
    }
}
fn expected(kind: Kind, labels: &[usize]) -> Vec<Call> {
    let values = match kind {
        Kind::Csi => vec![
            (vec![V::Param(1), V::Param(31)], ""),
            (vec![V::Param(0)], ""),
        ],
        Kind::Esc => vec![(vec![], "")],
        Kind::Osc => vec![(vec![], "foo=bar")],
        Kind::Dcs => vec![(vec![V::Param(1), V::Param(2), V::Param(3)], "abc")],
        Kind::Apc => vec![(vec![], "abc")],
    };
    values
        .into_iter()
        .flat_map(|(params, payload)| {
            labels
                .iter()
                .map(move |&label| Call(label, params.clone(), payload.into()))
        })
        .collect()
}

#[test]
fn original_custom_handler_bubbling_order_and_disposal_all_five_families() {
    for kind in [Kind::Esc, Kind::Csi, Kind::Osc, Kind::Dcs, Kind::Apc] {
        for (answers, labels) in [
            (vec![true, true], vec![1]),
            (vec![true, false], vec![1, 0]),
            (vec![true, true, false], vec![2, 1]),
            (vec![true, true, true], vec![2]),
            (vec![true, false, false], vec![2, 1, 0]),
        ] {
            let mut parser = EscapeSequenceParser::new();
            let calls = Calls::default();
            for (label, answer) in answers.into_iter().enumerate() {
                register(&mut parser, kind, label, answer, &calls);
            }
            assert!(parser.parse(&data(input(kind)), None).unwrap().is_none());
            assert_eq!(*calls.borrow(), expected(kind, &labels), "{kind:?}");
        }
        for times in [1, 2] {
            let mut parser = EscapeSequenceParser::new();
            let calls = Calls::default();
            register(&mut parser, kind, 0, true, &calls);
            let custom = register(&mut parser, kind, 1, true, &calls);
            for _ in 0..times {
                custom.dispose();
            }
            assert!(parser.parse(&data(input(kind)), None).unwrap().is_none());
            assert_eq!(
                *calls.borrow(),
                expected(kind, &[0]),
                "{kind:?} dispose{times}"
            );
        }
    }
}

#[test]
fn original_exact_latest_to_original_order() {
    for (kind, input) in [
        (Kind::Esc, "\x1b%G"),
        (Kind::Csi, "\x1b[0m"),
        (Kind::Osc, "\x1b]1;foo=bar\x1b\\"),
        (Kind::Dcs, "\x1bP1;2;3+pabc\x1b\\"),
        (Kind::Apc, "\x1b_+pabc\x1b\\"),
    ] {
        let mut parser = EscapeSequenceParser::new();
        let calls = Calls::default();
        for label in 1..=3 {
            register(&mut parser, kind, label, label == 1, &calls);
        }
        assert!(parser.parse(&data(input), None).unwrap().is_none());
        let order: Vec<_> = calls.borrow().iter().map(|call| call.0).collect();
        assert_eq!(order, [3, 2, 1], "{kind:?}");
    }
}

#[test]
fn original_csi_osc_set_clear_handlers() {
    for kind in [Kind::Csi, Kind::Osc] {
        let mut parser = EscapeSequenceParser::new();
        let calls = Calls::default();
        register(&mut parser, kind, 0, true, &calls);
        assert!(parser.parse(&data(INPUT), None).unwrap().is_none());
        assert_eq!(*calls.borrow(), expected(kind, &[0]));
        clear(&mut parser, kind);
        clear(&mut parser, kind);
        calls.borrow_mut().clear();
        assert!(parser.parse(&data(INPUT), None).unwrap().is_none());
        assert!(calls.borrow().is_empty());
    }
}
