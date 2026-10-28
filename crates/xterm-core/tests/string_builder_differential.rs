// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
use xterm_core::string_builder::{LimitedStringBuilder, StringBuilder};

fn units(text: &str) -> Vec<u16> {
    if text.is_empty() {
        return Vec::new();
    }
    text.split(',')
        .map(|unit| u16::from_str_radix(unit, 16).unwrap())
        .collect()
}

#[test]
fn matches_seeded_javascript_traces() {
    let mut previous = "";
    let mut plain = StringBuilder::new();
    let mut limited = LimitedStringBuilder::new(0);
    let mut count = 0;
    for (line, row) in include_str!("fixtures/string-builder-traces.tsv")
        .lines()
        .enumerate()
    {
        let columns: Vec<_> = row.split('\t').collect();
        assert_eq!(columns.len(), 7, "row {line}");
        let is_plain = columns[1] == "-";
        if columns[0] != previous {
            plain.reset();
            limited = if let Some(number) = columns[1].strip_prefix("j:") {
                LimitedStringBuilder::with_js_limit(number.parse().unwrap())
            } else {
                LimitedStringBuilder::new(if is_plain {
                    0
                } else {
                    columns[1].parse().unwrap()
                })
            };
            previous = columns[0];
        }
        let input = units(columns[3]);
        let overflow = match (is_plain, columns[2]) {
            (true, "reset") => {
                plain.reset();
                false
            }
            (false, "reset") => {
                limited.reset();
                false
            }
            (true, "append") => {
                plain.append_utf16(&input);
                false
            }
            (false, "append") => limited.append_utf16(&input),
            _ => panic!("invalid action on row {line}"),
        };
        let (length, actual) = if is_plain {
            (plain.len(), plain.as_utf16())
        } else {
            (limited.len(), limited.as_utf16())
        };
        assert_eq!(overflow, columns[4] == "1", "overflow on row {line}");
        assert_eq!(length, columns[5].parse::<usize>().unwrap(), "row {line}");
        assert_eq!(actual, units(columns[6]), "content on row {line}");
        count += 1;
    }
    assert_eq!(count, 16_648);
}

#[test]
fn preserves_split_pairs_and_lone_surrogates() {
    let mut b = StringBuilder::new();
    assert!(b.is_empty());
    b.append_utf16(&[0xd83d]);
    assert!(b.to_utf8().is_err());
    assert_eq!(b.as_utf16(), &[0xd83d]);
    b.append_utf16(&[0xde00]);
    assert_eq!(b.len(), 2);
    assert_eq!(b.to_utf8().unwrap(), "😀");
    b.append_utf16(&[0xdc00]);
    assert!(b.to_utf8().is_err());
    assert_eq!(b.into_utf16(), vec![0xd83d, 0xde00, 0xdc00]);
}

#[test]
fn utf8_helper_counts_utf16_units() {
    let mut b = StringBuilder::new();
    b.append("A😀\0e\u{301}");
    assert_eq!(b.len(), 6);
    assert_eq!(b.to_utf8().unwrap(), "A😀\0e\u{301}");
    let mut limited = LimitedStringBuilder::new(2);
    assert!(limited.is_empty());
    assert!(!limited.append("😀"));
    assert_eq!(limited.len(), 2);
    assert!(!limited.append(""));
    assert!(limited.append("x"));
    assert!(limited.is_empty());
    assert!(!limited.append("ok"));
    assert_eq!(limited.to_utf8().unwrap(), "ok");
}

#[test]
fn maximal_limit_does_not_wrap() {
    let mut b = LimitedStringBuilder::new(usize::MAX);
    assert!(!b.append_utf16(&[0xd800, 0]));
    assert_eq!(b.len(), 2);
    assert!(b.to_utf8().is_err());
    b.reset();
    assert!(b.is_empty());
}
