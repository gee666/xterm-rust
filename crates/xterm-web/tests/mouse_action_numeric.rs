// Copyright (c) 2017, 2026 The xterm.js authors. MIT license.
use std::convert::Infallible;
use xterm_core::mouse_state::{MouseAction, MouseButton, MouseEvent, MouseStateService};
use xterm_web::input::mouse_policy::{MouseOutput, MousePolicy};

fn event(button: f64, action: f64) -> MouseEvent {
    MouseEvent {
        col: 0.0,
        row: 0.0,
        x: 120.5,
        y: 1000.0,
        button: MouseButton::RawNumber(button),
        action: MouseAction::RawNumber(action),
        ctrl: Some(true),
        alt: Some(true),
        shift: Some(true),
    }
}

#[test]
fn normalized_reports_match_every_source_numeric_action_trace() {
    let corpus = include_str!("fixtures/mouse-raw-actions.tsv");
    assert_eq!(corpus.lines().count(), 2160);
    let service = MouseStateService::<()>::new();
    for (index, row) in corpus.lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 12, "row {index}");
        service.set_active_protocol(fields[0]).unwrap();
        service.set_active_encoding(fields[1]).unwrap();
        let mut event = event(fields[2].parse().unwrap(), fields[3].parse().unwrap());
        let action_bits = event.action.numeric_value().to_bits();
        let mut outputs = Vec::new();
        let result = MousePolicy::new()
            .trigger(&service, &mut event, 80.0, 24.0, |output| {
                outputs.push(output);
                Ok::<_, Infallible>(())
            })
            .unwrap();
        assert_eq!(result, fields[4] == "1", "row {index}: {row}");
        assert_eq!(event.col, fields[5].parse::<f64>().unwrap(), "row {index}");
        assert_eq!(event.row, fields[6].parse::<f64>().unwrap(), "row {index}");
        assert_eq!(event.ctrl, Some(fields[7] == "1"), "row {index}");
        assert_eq!(event.alt, Some(fields[8] == "1"), "row {index}");
        assert_eq!(event.shift, Some(fields[9] == "1"), "row {index}");
        assert_eq!(
            event.action.numeric_value().to_bits(),
            action_bits,
            "row {index}"
        );
        let units: Vec<u16> = if fields[11] == "-" {
            Vec::new()
        } else {
            fields[11]
                .split(',')
                .map(|unit| u16::from_str_radix(unit, 16).unwrap())
                .collect()
        };
        let expected = match fields[10] {
            "-" => Vec::new(),
            "binary" => vec![MouseOutput::Binary(units)],
            "data" => vec![MouseOutput::UserData(units)],
            other => panic!("invalid fixture output {other}"),
        };
        assert_eq!(outputs, expected, "row {index}: {row}");
    }
}

#[test]
fn raw_and_named_move_actions_share_debounce_and_preserve_fractional_moves() {
    let service = MouseStateService::<()>::new();
    service.set_active_protocol("ANY").unwrap();
    service.set_active_encoding("SGR").unwrap();
    let policy = MousePolicy::new();
    let mut first = event(0.0, 32.0);
    let mut outputs = Vec::new();
    assert!(policy
        .trigger(&service, &mut first, 80.0, 24.0, |out| {
            outputs.push(out);
            Ok::<_, Infallible>(())
        })
        .unwrap());
    let mut named = event(0.0, 32.0);
    named.action = MouseAction::Move;
    assert!(!policy
        .trigger(
            &service,
            &mut named,
            80.0,
            24.0,
            |_| -> Result<(), Infallible> { panic!("same raw/named move must debounce") }
        )
        .unwrap());
    let mut fractional = event(0.0, 32.5);
    assert!(policy
        .trigger(&service, &mut fractional, 80.0, 24.0, |out| {
            outputs.push(out);
            Ok::<_, Infallible>(())
        })
        .unwrap());
    assert_eq!(outputs.len(), 2);
    assert_eq!(
        outputs[0],
        MouseOutput::UserData("\x1b[<60;1;1M".encode_utf16().collect())
    );
    assert_eq!(
        outputs[1],
        MouseOutput::UserData("\x1b[<28;1;1M".encode_utf16().collect())
    );
}
