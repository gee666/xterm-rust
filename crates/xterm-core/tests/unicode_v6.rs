// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Port of UnicodeV6.test.ts assertions.

use xterm_core::unicode::{UnicodeCharProperties as Properties, UnicodeVersionProvider};
use xterm_core::unicode_v6::UnicodeV6;

fn expected_widths() -> Vec<u8> {
    let mut widths = vec![u8::MAX; 0x110000];
    let mut previous_end = 0;
    for line in include_str!("fixtures/unicode_v6_widths.txt").lines() {
        let values: Vec<_> = line.split_whitespace().collect();
        assert_eq!(values.len(), 3);
        let start = usize::from_str_radix(values[0], 16).unwrap();
        let end = usize::from_str_radix(values[1], 16).unwrap();
        let width: u8 = values[2].parse().unwrap();
        assert_eq!(start, previous_end, "fixture must cover every codepoint");
        assert!(end > start && end <= widths.len());
        assert!(width <= 2);
        widths[start..end].fill(width);
        previous_end = end;
    }
    assert_eq!(previous_end, widths.len());
    widths
}

#[test]
fn wcwidth_should_match_all_values_from_the_old_implementation() {
    // The generator runs the original old-vs-new assertion over the full BMP.
    // This fixture contains the exact new implementation results from that run.
    let provider = UnicodeV6::new();
    let widths = expected_widths();
    for (codepoint, expected) in widths.iter().enumerate().take(65536) {
        assert_eq!(
            provider.wcwidth(codepoint as u32).columns(),
            *expected,
            "mismatch for i: {codepoint}"
        );
    }
}

#[test]
fn supplementary_and_surrogate_codepoints_match_javascript() {
    let provider = UnicodeV6::new();
    let widths = expected_widths();
    for (codepoint, expected) in widths.iter().enumerate() {
        assert_eq!(
            provider.wcwidth(codepoint as u32).columns(),
            *expected,
            "U+{codepoint:04X}"
        );
    }
    // Upstream does not reject numeric values beyond the Unicode scalar range.
    assert_eq!(provider.wcwidth(0x110000).columns(), 1);
    assert_eq!(provider.wcwidth(u32::MAX).columns(), 1);
}

#[test]
fn packed_properties_match_javascript_for_all_width_states() {
    let provider = UnicodeV6::new();
    assert_eq!(provider.version(), "6");
    for line in include_str!("fixtures/unicode_v6_properties.txt").lines() {
        let values: Vec<_> = line.split_whitespace().collect();
        let codepoint = u32::from_str_radix(values[0], 16).unwrap();
        let preceding = u32::from_str_radix(values[1], 16).unwrap();
        let expected: u32 = values[2].parse().unwrap();
        assert_eq!(
            provider
                .char_properties(codepoint, Properties::from_raw(preceding))
                .raw(),
            expected,
            "codepoint {codepoint:x}, preceding {preceding:x}"
        );
    }
}

#[test]
fn property_construction_masks_state_and_width_like_javascript() {
    for state in [0, 1, 0xffffff, 0x1000000, u32::MAX] {
        for width in 0..=255 {
            for join in [false, true] {
                let value = Properties::new(state, width, join);
                assert_eq!(value.width(), (width & 3) as u8);
                assert_eq!(value.char_kind(), (state & 0xffffff) as i32);
                assert_eq!(value.should_join(), join);
            }
        }
    }
    assert_eq!(Properties::from_raw(u32::MAX).char_kind(), -1);
    assert_eq!(Properties::from_raw(0x80000000).char_kind(), -268435456);
    assert_eq!(Properties::default().raw(), 0);
}

#[test]
fn lazy_table_initialization_is_shared_and_thread_safe() {
    let threads: Vec<_> = (0..8)
        .map(|_| {
            std::thread::spawn(|| {
                let provider = UnicodeV6;
                assert_eq!(provider.wcwidth(0x302a).columns(), 0);
                assert_eq!(provider.wcwidth(0x4e00).columns(), 2);
                assert_eq!(provider.wcwidth(0x303f).columns(), 1);
            })
        })
        .collect();
    for thread in threads {
        thread.join().unwrap();
    }
}
