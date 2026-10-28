// Copyright (c) 2019, 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
// Independent consumer of the unchanged-source oracle from task #56.
use std::collections::BTreeSet;
use xterm_core::mouse_state::{MouseAction, MouseButton, MouseEvent, MouseStateService};

fn units(value: &str) -> Vec<u16> {
    if value == "-" {
        return Vec::new();
    }
    value
        .split(',')
        .map(|unit| u16::from_str_radix(unit, 16).unwrap())
        .collect()
}

fn flag(value: &str) -> Option<bool> {
    match value {
        "0" => Some(false),
        "1" => Some(true),
        _ => panic!("invalid generated modifier: {value}"),
    }
}

fn compare_corpus(corpus: &str, expected_count: usize, require_surrogates: bool) {
    let service = MouseStateService::<()>::new();
    let mut combinations = BTreeSet::new();
    let mut compared = 0;
    let mut surrogate_outputs = 0;
    for (index, row) in corpus.lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 16, "fixture row {index}");
        let mut event = MouseEvent {
            col: fields[2].parse().unwrap(),
            row: fields[3].parse().unwrap(),
            x: fields[4].parse().unwrap(),
            y: fields[5].parse().unwrap(),
            button: MouseButton::Raw(fields[6].parse().unwrap()),
            action: MouseAction::try_from(fields[7].parse::<u8>().unwrap()).unwrap(),
            ctrl: flag(fields[8]),
            alt: flag(fields[9]),
            shift: flag(fields[10]),
        };
        let before = [event.col, event.row, event.x, event.y].map(f64::to_bits);
        service.set_active_protocol(fields[0]).unwrap();
        service.set_active_encoding(fields[1]).unwrap();
        let accepted = service.restrict_mouse_event(&mut event);
        assert_eq!(
            accepted,
            fields[11] == "1",
            "restriction row {index}: {row}"
        );
        assert_eq!(event.ctrl, flag(fields[12]), "ctrl row {index}: {row}");
        assert_eq!(event.alt, flag(fields[13]), "alt row {index}: {row}");
        assert_eq!(event.shift, flag(fields[14]), "shift row {index}: {row}");
        assert_eq!(
            [event.col, event.row, event.x, event.y].map(f64::to_bits),
            before,
            "coordinate mutation on row {index}"
        );
        // Encoding is independently callable upstream, even for rejected events.
        let actual = service.encode_mouse_event(&mut event);
        let expected = units(fields[15]);
        assert_eq!(actual, expected, "encoding row {index}: {row}");
        surrogate_outputs +=
            usize::from(expected.iter().any(|unit| (0xd800..=0xdfff).contains(unit)));
        combinations.insert((fields[0], fields[1]));
        compared += 1;
    }
    assert_eq!(
        compared, expected_count,
        "oracle inventory must not shrink silently"
    );
    assert_eq!(
        combinations.len(),
        15,
        "all five protocols and three encodings"
    );
    if require_surrogates {
        assert!(
            surrogate_outputs > 0,
            "DEFAULT must preserve lone UTF-16 surrogates"
        );
    }
}

#[test]
fn matches_every_source_protocol_encoding_and_mutation_trace() {
    compare_corpus(include_str!("fixtures/mouse-state.tsv"), 12_226, true);
}

#[test]
fn raw_signed_buttons_match_source_bitwise_and_wheel_comparisons() {
    compare_corpus(include_str!("fixtures/mouse-raw-buttons.tsv"), 4_200, false);
    assert_eq!(MouseButton::Raw(4), MouseButton::Wheel);
    assert_eq!(MouseButton::Raw(3), MouseButton::None);
    assert_eq!(MouseButton::Raw(0), MouseButton::Left);
    assert_ne!(MouseButton::Raw(-1), MouseButton::None);
}
