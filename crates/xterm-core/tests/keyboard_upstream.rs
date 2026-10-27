// Copyright (c) 2014 The xterm.js authors. All rights reserved.
// Licensed under MIT. All Keyboard.test.ts assertions are retained as fixtures.
use xterm_core::keyboard::{evaluate_keyboard_event, KeyboardEvent};

fn units(value: &str) -> Vec<u16> {
    value
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| u16::from_str_radix(s, 16).expect("generated UTF-16 fixture"))
        .collect()
}

#[test]
fn every_upstream_keyboard_assertion() {
    let mut assertions = 0;
    for line in include_str!("fixtures/keyboard_upstream.tsv").lines() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 9, "fixture shape");
        let key = units(fields[1]);
        let code = units(fields[2]);
        let modifiers: u8 = fields[3].parse().expect("fixture modifiers");
        let options: u8 = fields[4].parse().expect("fixture options");
        let event = KeyboardEvent {
            key_code: fields[0].parse().expect("fixture keycode"),
            key: &key,
            code: &code,
            shift_key: modifiers & 1 != 0,
            alt_key: modifiers & 2 != 0,
            ctrl_key: modifiers & 4 != 0,
            meta_key: modifiers & 8 != 0,
        };
        let result =
            evaluate_keyboard_event(&event, options & 1 != 0, options & 2 != 0, options & 4 != 0);
        assert_eq!(
            result.result_type as u8,
            fields[5].parse::<u8>().unwrap(),
            "{}",
            fields[8]
        );
        assert_eq!(result.cancel, fields[6] == "1", "{}", fields[8]);
        let expected = (fields[7] != "-").then(|| units(fields[7]));
        assert_eq!(result.key, expected, "{}", fields[8]);
        assertions += 1;
    }
    assert_eq!(assertions, 156, "original assertion count must not shrink");
}
