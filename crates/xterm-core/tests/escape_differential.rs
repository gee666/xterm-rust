// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Replay traces from the unchanged upstream parser.
use std::{cell::RefCell, rc::Rc};
use xterm_core::parser::{
    apc::ApcAction,
    dcs::DcsAction,
    escape::EscapeSequenceParser,
    osc::OscAction,
    params::{ParamValue, Params},
    table::ParserState,
};

fn hex(values: impl IntoIterator<Item = u32>) -> String {
    values
        .into_iter()
        .map(|value| format!("{value:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
fn params(value: &Params) -> String {
    value
        .to_array()
        .iter()
        .map(|value| match value {
            ParamValue::Param(value) => format!("p{value}"),
            ParamValue::SubParams(values) => format!(
                "s{}",
                values
                    .iter()
                    .map(i32::to_string)
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        })
        .collect::<Vec<_>>()
        .join(":")
}
type Reports = Rc<RefCell<Vec<String>>>;
fn instrument() -> (EscapeSequenceParser, Reports) {
    let mut parser = EscapeSequenceParser::new();
    let reports: Reports = Rc::default();
    let target = Rc::clone(&reports);
    parser.set_print_handler(move |data| {
        target
            .borrow_mut()
            .push(format!("P:{}", hex(data.iter().copied())))
    });
    let target = Rc::clone(&reports);
    parser.set_execute_handler_fallback(move |code| target.borrow_mut().push(format!("X:{code}")));
    let target = Rc::clone(&reports);
    parser.set_csi_handler_fallback(move |id, values| {
        target
            .borrow_mut()
            .push(format!("C:{id}:{}", params(values)))
    });
    let target = Rc::clone(&reports);
    parser.set_esc_handler_fallback(move |id| target.borrow_mut().push(format!("E:{id}")));
    let target = Rc::clone(&reports);
    parser.set_error_handler(move |state| {
        target.borrow_mut().push(format!(
            "R:{}:{}:{}:{}:{}",
            state.position,
            state.code,
            state.current_state as u8,
            state.collect,
            params(state.params)
        ));
        false
    });
    let target = Rc::clone(&reports);
    parser.osc.set_handler_fallback(move |id, action| {
        let action = match action {
            OscAction::Start => "S".to_owned(),
            OscAction::Put(data) => format!("P:{}", hex(data.into_iter().map(u32::from))),
            OscAction::End(success) => format!("E:{}", u8::from(success)),
        };
        target.borrow_mut().push(format!("O:{id}:{action}"));
    });
    let target = Rc::clone(&reports);
    parser.dcs.set_handler_fallback(move |id, action| {
        let action = match action {
            DcsAction::Hook(values) => format!("H:{}", params(&values)),
            DcsAction::Put(data) => format!("P:{}", hex(data.into_iter().map(u32::from))),
            DcsAction::Unhook(success) => format!("U:{}", u8::from(success)),
        };
        target
            .borrow_mut()
            .push(format!("D:{}:{action}", id as u32));
    });
    let target = Rc::clone(&reports);
    parser.apc.set_handler_fallback(move |id, action| {
        let action = match action {
            ApcAction::Start => "S".to_owned(),
            ApcAction::Put(data) => format!("P:{}", hex(data.into_iter().map(u32::from))),
            ApcAction::End(success) => format!("E:{}", u8::from(success)),
        };
        target.borrow_mut().push(format!("A:{id}:{action}"));
    });
    (parser, reports)
}

#[test]
fn all_states_and_seeded_streams_match_original_snapshots_and_callbacks() {
    let fixture = include_str!("fixtures/escape-parser.tsv");
    let mut count = 0;
    for (row, line) in fixture.lines().enumerate() {
        let (operations, snapshots) = line.split_once('\t').unwrap();
        let snapshots: Vec<_> = snapshots.split('~').collect();
        let (mut parser, reports) = instrument();
        for (step, operation) in operations.split('|').enumerate() {
            reports.borrow_mut().clear();
            match operation.as_bytes()[0] {
                b'r' => parser.reset().unwrap(),
                b's' => {
                    parser.current_state =
                        ParserState::from_code(operation[1..].parse().unwrap()).unwrap()
                }
                b'j' => parser
                    .preceding_join_state
                    .set(operation[1..].parse().unwrap()),
                b'p' => {
                    let data: Vec<_> = operation[1..]
                        .split(',')
                        .filter(|s| !s.is_empty())
                        .map(|s| u32::from_str_radix(s, 16).unwrap())
                        .collect();
                    assert!(
                        parser.parse(&data, None).unwrap().is_none(),
                        "unexpected pending at row {row} step {step}"
                    );
                }
                _ => panic!("invalid fixture operation {operation}"),
            }
            let actual = format!(
                "{}/{}/{}/{}/{}",
                parser.current_state as u8,
                parser.collect(),
                params(parser.params()),
                parser.preceding_join_state.get(),
                reports.borrow().join("|")
            );
            assert_eq!(actual, snapshots[step], "seed=0xe5ca9e01 row={row} step={step} operation={operation}; retained trace={operations}");
        }
        count += 1;
    }
    assert_eq!(count, 3334);
}
