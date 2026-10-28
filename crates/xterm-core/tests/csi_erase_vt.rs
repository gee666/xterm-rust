// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
//! Terminal2.test.ts erase fixtures at 80x25. Explicit ONLCR modeling is not an
//! observed native PTY/browser run. EL retains its original legacy-xterm skip,
//! while every fixture, including EL, must match the unchanged real JS engine.
use xterm_core::terminal::CoreTerminal;
const CASES: [(&str, &[u8], &str); 5] = [
    (
        "t0054-ECH",
        include_bytes!("fixtures/erase-vt/t0054-ECH.in"),
        include_str!("fixtures/erase-vt/t0054-ECH.text"),
    ),
    (
        "t0055-EL",
        include_bytes!("fixtures/erase-vt/t0055-EL.in"),
        include_str!("fixtures/erase-vt/t0055-EL.text"),
    ),
    (
        "t0056-ED",
        include_bytes!("fixtures/erase-vt/t0056-ED.in"),
        include_str!("fixtures/erase-vt/t0056-ED.text"),
    ),
    (
        "t0057-ED3",
        include_bytes!("fixtures/erase-vt/t0057-ED3.in"),
        include_str!("fixtures/erase-vt/t0057-ED3.text"),
    ),
    (
        "t0091-alt_screen_ED3",
        include_bytes!("fixtures/erase-vt/t0091-alt_screen_ED3.in"),
        include_str!("fixtures/erase-vt/t0091-alt_screen_ED3.text"),
    ),
];
fn normalize_expected(expected: &str) -> String {
    expected
        .split('\n')
        .map(str::trim_end)
        .collect::<Vec<_>>()
        .join("\n")
}
fn viewport(input: &[u8]) -> String {
    let mut data = Vec::new();
    for &byte in input {
        if byte == b'\n' {
            data.push(b'\r');
        }
        data.push(byte);
    }
    let mut terminal = CoreTerminal::new(80, 25, 1000).unwrap();
    terminal.write_bytes(&data).unwrap();
    let active = terminal.state().active_buffer();
    let b = active.borrow();
    let mut actual = String::new();
    for y in b.ybase..b.ybase + 25 {
        let line = b.lines.get(y).unwrap();
        let line = line.borrow();
        let text = String::from_utf16(&line.translate_to_string(true, None, None, None)).unwrap();
        actual.push_str(text.trim_end());
        actual.push('\n');
    }
    actual
}
#[test]
fn original_applicable_erase_vt_text_assertions_with_explicit_onlcr() {
    let mut count = 0;
    for (name, input, expected) in CASES.into_iter().filter(|case| case.0 != "t0055-EL") {
        assert_eq!(
            viewport(input),
            normalize_expected(expected),
            "unchanged upstream fixture {name}"
        );
        count += 1;
    }
    assert_eq!(count, 4);
}
#[test]
fn all_five_erase_vt_inputs_match_unchanged_js_including_upstream_skipped_el() {
    let mut count = 0;
    for row in include_str!("fixtures/erase-vt.tsv").lines() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 4);
        let (_, input, legacy) = CASES.iter().find(|case| case.0 == fields[0]).unwrap();
        let units: Vec<u16> = fields[1]
            .split(',')
            .map(|s| u16::from_str_radix(s, 16).unwrap())
            .collect();
        let expected = String::from_utf16(&units).unwrap();
        let actual = viewport(input);
        assert_eq!(actual, expected, "real JS output for {}", fields[0]);
        assert_eq!(
            actual == normalize_expected(legacy),
            fields[2] == "1",
            "retain legacy-match result for {}",
            fields[0]
        );
        assert_eq!(fields[3] == "1", fields[0] == "t0055-EL");
        count += 1;
    }
    assert_eq!(count, 5);
}
