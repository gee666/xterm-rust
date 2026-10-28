// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Private-method source oracle, not browser integration.
use serde_json::json;
use xterm_web::accessibility::{Announcements, CharacterAction, LocalizableStrings};
fn units(value: &str) -> Vec<u16> {
    value
        .split(',')
        .filter(|value| !value.is_empty())
        .map(|value| u16::from_str_radix(value, 16).unwrap())
        .collect()
}
#[test]
fn every_bmp_key_matches_the_original_unicode_control_regex() {
    let encoded = include_str!("fixtures/accessibility-control.hex").trim();
    let bytes = encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
        .collect::<Vec<_>>();
    assert_eq!(bytes.len(), 8192);
    for unit in 0..=u16::MAX {
        let mut state = Announcements::new();
        state.key(&[unit]).unwrap();
        let rejected = bytes[usize::from(unit) / 8] & (1 << (unit % 8)) != 0;
        assert_eq!(state.queued_keys().is_empty(), rejected, "key unit{unit:x}");
    }
}
#[test]
fn directed_and_seeded_announcements_match_every_original_state_transition() {
    let mut cases = 0;
    for (case, line) in include_str!("fixtures/accessibility-state.tsv")
        .lines()
        .enumerate()
    {
        let (operations, expected) = line.split_once('\t').unwrap();
        let mut state = Announcements::new();
        let labels = LocalizableStrings::new();
        let mut live = Vec::new();
        let mut snapshots = Vec::new();
        for operation in operations.split(';') {
            let (kind, value) = operation.split_once(':').unwrap_or((operation, ""));
            match kind {
                "c" => {
                    if state.character(&units(value)).unwrap() == CharacterAction::Overflow {
                        live = labels.too_much_output().unwrap();
                    }
                }
                "k" => {
                    live.clear();
                    state.key(&units(value)).unwrap();
                }
                "l" => labels.set_too_much_output(units(value)),
                "p" => live = units(value),
                "t" => {
                    for _ in 0..value.parse::<usize>().unwrap() {
                        assert_eq!(state.character(&[32]).unwrap(), CharacterAction::Unchanged);
                    }
                }
                "b" => {
                    live.clear();
                    state.clear_live_region();
                }
                "f" => {
                    if let Some(text) = state
                        .flush(&live, &labels.too_much_output().unwrap())
                        .unwrap()
                    {
                        live = text;
                    }
                }
                _ => panic!("unknown oracle operation {operation}"),
            }
            snapshots.push(json!({ "live": &live, "pending": state.pending(), "consume": state.queued_keys(), "lines": state.line_count() }));
        }
        let expected: serde_json::Value = serde_json::from_str(expected).unwrap();
        assert_eq!(
            json!(snapshots),
            expected,
            "seed0x0a11e72 case{case}: {operations}"
        );
        cases += 1;
    }
    assert_eq!(cases, 268);
}
