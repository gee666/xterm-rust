// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{headless::Terminal, terminal::TerminalEvent};

fn snapshot(terminal: &Terminal) -> String {
    terminal
        .with_state(|state| {
            let buffer = state.active_buffer();
            let buffer = buffer.borrow();
            let line = buffer.lines.get(0).unwrap();
            let text = line.borrow().translate_to_string(true, None, None, None);
            let hex = text
                .iter()
                .map(|unit| format!("{unit:x}"))
                .collect::<Vec<_>>()
                .join(",");
            format!("{},{}|{hex}", buffer.x, buffer.y)
        })
        .unwrap()
}

fn run_case(case: &str) {
    let rows: Vec<_> = include_str!("fixtures/public-api-integration.tsv")
        .lines()
        .filter(|row| row.starts_with(&format!("{case}\t")))
        .collect();
    assert_eq!(rows.len(), 1, "fixture case {case}");
    for row in rows {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 4);
        let name = fields[0];
        let input: Vec<u16> = fields[1]
            .split(',')
            .map(|unit| u16::from_str_radix(unit, 16).unwrap())
            .collect();
        let expected_observations = fields[2];
        let expected_final = fields[3];
        let terminal = Rc::new(Terminal::new(5, 3, 1000).unwrap());
        let weak = Rc::downgrade(&terminal);
        let observations = Rc::new(RefCell::new(Vec::new()));
        let output = Rc::clone(&observations);
        terminal.on_event().subscribe(move |event| {
            let selected = match name {
                "bell_observe" | "bell_reset" => matches!(event, TerminalEvent::Bell),
                "data_observe" => matches!(event, TerminalEvent::Data(_)),
                "linefeed_observe" => matches!(event, TerminalEvent::LineFeed),
                "title_observe" => matches!(event, TerminalEvent::TitleChange(_)),
                other => panic!("unknown fixture {other}"),
            };
            if selected {
                let terminal = weak.upgrade().unwrap();
                output.borrow_mut().push(snapshot(&terminal));
                if name == "bell_reset" {
                    terminal.reset().unwrap();
                }
            }
        });
        terminal.write_utf16(&input).unwrap();
        assert_eq!(
            observations.borrow().join("/"),
            expected_observations,
            "event time for {name}"
        );
        assert_eq!(
            snapshot(&terminal),
            expected_final,
            "final state for {name}"
        );
        terminal.dispose().unwrap();
    }
}

#[test]
fn pa04_bell_observes_state_before_following_print() {
    run_case("bell_observe");
}
#[test]
fn pa04_bell_reset_precedes_following_print() {
    run_case("bell_reset");
}
#[test]
fn pa04_data_reply_observes_state_before_following_print() {
    run_case("data_observe");
}
#[test]
fn pa04_linefeed_observes_state_before_following_print() {
    run_case("linefeed_observe");
}
#[test]
fn pa04_title_observes_state_before_following_print() {
    run_case("title_observe");
}
