// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
use xterm_core::string_builder::{LimitedStringBuilder, PayloadLimit, StringBuilder};

#[test]
fn every_single_utf16_unit_survives_observation() {
    let mut builder = StringBuilder::new();
    for unit in 0..=u16::MAX {
        builder.reset();
        builder.append_utf16(&[unit]);
        assert_eq!(builder.len(), 1);
        assert_eq!(builder.as_utf16(), [unit]);
        assert_eq!(builder.as_utf16(), [unit], "observation cannot mutate");
        if (0xd800..=0xdfff).contains(&unit) {
            assert!(builder.to_utf8().is_err());
            assert_eq!(builder.as_utf16(), [unit]);
        } else {
            assert_eq!(
                builder
                    .to_utf8()
                    .unwrap()
                    .encode_utf16()
                    .collect::<Vec<_>>(),
                [unit]
            );
        }
    }
}

#[test]
fn every_chunk_partition_preserves_surrogate_sequences() {
    for units in [
        vec![0xd83d, 0xde00],
        vec![0xd800, 0xd800, 0xdc00],
        vec![0xdc00, 0xd800],
        vec![0xdc00, 0xdfff],
        vec![0xd800, 0, 0xdc00],
        vec![0x41, 0xd83d, 0xde00, 0, 0x65, 0x301],
    ] {
        for mask in 0..1 << (units.len() - 1) {
            let mut builder = StringBuilder::new();
            let mut start = 0;
            for end in 1..=units.len() {
                if end == units.len() || mask & (1 << (end - 1)) != 0 {
                    builder.append_utf16(&[]);
                    builder.append_utf16(&units[start..end]);
                    builder.append_utf16(&[]);
                    start = end;
                }
            }
            assert_eq!(builder.len(), units.len());
            assert_eq!(builder.as_utf16(), units);
            builder.reset();
            builder.append_utf16(&[0x61]);
            assert_eq!(builder.as_utf16(), [0x61]);
        }
    }
}

#[test]
fn boundary_matrix_matches_single_and_cumulative_utf8_and_utf16() {
    for limit in [0, 1, 2, 3, 5, 100] {
        for length in 0..=limit + 1 {
            let units = vec![u16::from(b'A'); length];
            let string = "A".repeat(length);
            for raw in [false, true] {
                let mut builder = LimitedStringBuilder::new(limit);
                let overflow = if raw {
                    builder.append_utf16(&units)
                } else {
                    builder.append(&string)
                };
                assert_eq!(overflow, length > limit);
                assert_eq!(builder.len(), if overflow { 0 } else { length });
                assert_eq!(builder.as_utf16(), if overflow { &[] } else { &units[..] });
                assert!(!builder.append(""));
                builder.reset();
                for position in 0..length {
                    assert!(!builder.append_utf16(&[]));
                    let overflow = if raw {
                        builder.append_utf16(&[u16::from(b'A')])
                    } else {
                        builder.append("A")
                    };
                    assert_eq!(overflow, position >= limit);
                }
                assert_eq!(
                    builder.as_utf16(),
                    if length > limit { &[] } else { &units[..] }
                );
                assert_eq!(builder.limit(), PayloadLimit::Units(limit));
            }
        }
        let mut raw = LimitedStringBuilder::new(limit);
        let mut utf8 = LimitedStringBuilder::new(limit);
        assert_eq!(raw.append_utf16(&[0xd83d, 0xde00]), utf8.append("😀"));
        assert_eq!(raw.as_utf16(), utf8.as_utf16());
    }
}

#[test]
fn breach_never_poisons_future_payload_or_retains_surrogates() {
    let mut builder = LimitedStringBuilder::new(3);
    for _ in 0..2 {
        assert!(builder.append_utf16(&[0xd800, 0, 0xdc00, 0xffff]));
        assert!(builder.is_empty());
    }
    assert!(!builder.append("abc"));
    assert!(builder.append("x"));
    assert!(builder.is_empty());
    assert!(!builder.append("ab"));
    assert_eq!(builder.to_utf8().unwrap(), "ab");
    builder.reset();
    builder.reset();
    assert!(!builder.append(""));
    assert!(!builder.append("new"));
    assert_eq!(builder.to_utf8().unwrap(), "new");
}

#[test]
fn numeric_constructor_metadata_survives_every_operation() {
    for limit in [
        -0.0,
        0.0,
        0.5,
        1.0,
        1.5,
        -0.5,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        f64::MIN_POSITIVE,
    ] {
        let mut builder = LimitedStringBuilder::with_js_limit(limit);
        for chunk in ["", "a", "b", "", "😀"] {
            let length = builder.len() + chunk.encode_utf16().count();
            let expected_overflow = length as f64 > limit;
            assert_eq!(builder.append(chunk), expected_overflow);
            let PayloadLimit::JavaScript(actual) = builder.limit() else {
                panic!("lost numeric limit")
            };
            assert_eq!(actual.to_bits(), limit.to_bits());
        }
        builder.reset();
        let PayloadLimit::JavaScript(actual) = builder.limit() else {
            panic!("lost numeric limit")
        };
        assert_eq!(actual.to_bits(), limit.to_bits());
    }
}

#[test]
fn owned_snapshots_and_clone_are_independent_of_reset_and_reuse() {
    let mut builder = StringBuilder::new();
    builder.append("old");
    let owned = builder.clone().into_utf16();
    builder.reset();
    builder.reset();
    builder.append("new");
    assert_eq!(owned, "old".encode_utf16().collect::<Vec<_>>());
    assert_eq!(builder.to_utf8().unwrap(), "new");
    let mut bounded = LimitedStringBuilder::new(3);
    bounded.append("old");
    let snapshot = bounded.clone();
    bounded.reset();
    assert_eq!(snapshot.to_utf8().unwrap(), "old");
    assert!(bounded.is_empty());
    assert_eq!(bounded.limit(), PayloadLimit::Units(3));
}
