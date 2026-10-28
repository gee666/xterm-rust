// Licensed under MIT. Replay unchanged-source getter/throw traces in Rust.
#[path = "support/keyboard_live.rs"]
mod support;
use std::rc::Rc;
use support::{units, Source};
use xterm_core::input::{
    kitty_keyboard::{try_evaluate, EventType},
    win32_input_mode::Win32InputMode,
};

#[test]
fn complete_protocol_getter_traces_and_exact_error_identity_match_source() {
    let fixture = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/protocol-live.tsv"
    ))
    .expect("generate reviewed fixture with scripts/protocol-live-oracle.mjs --generate");
    let mut count = 0;
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 17, "{line}");
        let source = Source::from_fields(&fields, 6, 5);
        let result = match fields[1] {
            "K" => try_evaluate(
                &source,
                fields[2].parse().unwrap(),
                match fields[3] {
                    "1" => EventType::Press,
                    "2" => EventType::Repeat,
                    "3" => EventType::Release,
                    _ => panic!("invalid fixture event type"),
                },
                fields[4] == "1",
            ),
            "W" => Win32InputMode.try_evaluate_keyboard_event(&source, fields[3] == "1"),
            _ => panic!("invalid fixture encoder"),
        };
        assert_eq!(
            *source.reads.borrow(),
            fields[13],
            "{} getter order",
            fields[0]
        );
        if fields[14] == "throw" {
            assert!(
                Rc::ptr_eq(&result.unwrap_err(), &source.marker),
                "{} marker identity",
                fields[0]
            );
        } else {
            let result = result.unwrap();
            assert_eq!(
                result.result_type as u8,
                fields[14].parse::<u8>().unwrap(),
                "{} result kind",
                fields[0]
            );
            assert_eq!(
                result.cancel,
                fields[15] == "1",
                "{} cancellation",
                fields[0]
            );
            assert_eq!(
                result.key,
                (fields[16] != "!").then(|| units(fields[16])),
                "{} UTF16 payload",
                fields[0]
            );
        }
        count += 1;
    }
    assert_eq!(
        count, 78_692,
        "review source corpus changes before changing the pinned count"
    );
}
