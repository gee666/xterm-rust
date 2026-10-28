// Copyright (c) 2017-2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. Direct AccessibilityManager state and numeric assertions.
use xterm_web::accessibility::{
    map_point, selection_range, Announcements, CellPoint, CharacterAction, LocalizableStrings,
};
fn units(value: &str) -> Vec<u16> {
    value.encode_utf16().collect()
}

#[test]
fn every_bmp_unit_matches_control_key_suppression_and_utf16_echo() {
    for unit in 0..=u16::MAX {
        let mut state = Announcements::new();
        state.key(&[unit]).unwrap();
        let control = matches!(unit, 0..=0x1f | 0x7f..=0x9f);
        assert_eq!(
            state.queued_keys().len(),
            usize::from(!control),
            "unit{unit:x}"
        );
        assert_eq!(
            state.character(&[unit]).unwrap(),
            CharacterAction::Unchanged
        );
        assert!(state.queued_keys().is_empty());
        let expected = if control { vec![unit] } else { Vec::new() };
        assert_eq!(state.pending(), expected);
        assert_eq!(state.line_count(), usize::from(unit == 10));
    }
}
#[test]
fn echo_consumes_one_complete_string_and_mismatch_still_announces() {
    let mut state = Announcements::new();
    for key in [
        vec![],
        units("ab"),
        vec![0xd83d, 0xde00],
        vec![0xd800],
        units("X"),
    ] {
        state.key(&key).unwrap();
    }
    state.character(&[]).unwrap();
    state.character(&units("a")).unwrap();
    state.character(&[0xd83d, 0xde00]).unwrap();
    state.character(&[0xd800]).unwrap();
    state.character(&units("Y")).unwrap();
    state.character(&units("Z")).unwrap();
    assert_eq!(state.pending(), units("aYZ"));
    assert!(state.queued_keys().is_empty());
}
#[test]
fn key_and_blur_clear_only_line_count_not_pending_or_echo_queue() {
    let mut state = Announcements::new();
    state.character(&units("out")).unwrap();
    state.character(&[10]).unwrap();
    state.key(&units("one")).unwrap();
    state.key(&units("two")).unwrap();
    state.clear_live_region();
    assert_eq!(state.line_count(), 0);
    assert_eq!(state.pending(), units("out\n"));
    assert_eq!(state.queued_keys().len(), 2);
    state.character(&units("one")).unwrap();
    let mut snapshot = state.pending_snapshot().unwrap();
    snapshot.clear();
    assert_eq!(
        state.flush(&[], &units("overflow")).unwrap(),
        Some(units("out\n"))
    );
    assert!(state.pending().is_empty());
    assert_eq!(state.queued_keys().len(), 1);
}
#[test]
fn exact_newline21_limit_and_render_clear_transition() {
    let mut state = Announcements::new();
    for index in 1..=21 {
        assert_eq!(
            state.character(&[10]).unwrap(),
            if index == 21 {
                CharacterAction::Overflow
            } else {
                CharacterAction::Unchanged
            }
        );
        assert_eq!(state.line_count(), index);
    }
    state.character(&units("ignored")).unwrap();
    assert_eq!(state.pending(), &[10; 21]);
    let label = units("overflow");
    assert_eq!(state.flush(&label, &label).unwrap(), Some(vec![10; 21]));
    assert_eq!(state.line_count(), 0);
    state.character(&units("accepted")).unwrap();
    assert_eq!(state.flush(&[], &label).unwrap(), Some(units("accepted")));
}
#[test]
fn changed_overflow_label_does_not_reset_limit_and_empty_label_uses_equality() {
    let mut state = Announcements::new();
    for _ in 0..21 {
        state.character(&[10]).unwrap();
    }
    let old = units("old");
    let new = units("new");
    let mut expected = old.clone();
    expected.extend([10; 21]);
    assert_eq!(state.flush(&old, &new).unwrap(), Some(expected));
    assert_eq!(state.line_count(), 21);
    state.character(&units("dropped")).unwrap();
    assert!(state.pending().is_empty());
    state.clear_live_region();
    state.character(&[10]).unwrap();
    assert_eq!(state.flush(&[], &[]).unwrap(), Some(vec![10]));
    assert_eq!(state.line_count(), 0);
    assert_eq!(
        state
            .flush(&units("untouched"), &units("untouched"))
            .unwrap(),
        None
    );
}
#[test]
fn embedded_newlines_and_repeated_spaces_are_not_linefeed_events() {
    let mut state = Announcements::new();
    state.character(&units("a\nb")).unwrap();
    for _ in 0..80 {
        state.character(&[32]).unwrap();
    }
    assert_eq!(state.line_count(), 0);
    let mut expected = units("a\nb");
    expected.extend([32; 80]);
    assert_eq!(state.pending(), expected);
}
#[test]
fn localized_defaults_aliasing_utf16_and_independent_runtimes() {
    let strings = LocalizableStrings::new();
    let clone = strings.clone();
    assert_eq!(strings.prompt_label().unwrap(), units("Terminal input"));
    assert_eq!(
        strings.too_much_output().unwrap(),
        units("Too much output to announce, navigate to rows manually to read")
    );
    clone.set_prompt_label(vec![0xd800, 0, 0xfeff]);
    clone.set_too_much_output(vec![0xd83d, 0xde00]);
    assert_eq!(strings.prompt_label().unwrap(), [0xd800, 0, 0xfeff]);
    assert_eq!(strings.too_much_output().unwrap(), [0xd83d, 0xde00]);
    let mut copy = strings.prompt_label().unwrap();
    copy.clear();
    assert_eq!(strings.prompt_label().unwrap(), [0xd800, 0, 0xfeff]);
    assert_eq!(
        LocalizableStrings::new().prompt_label().unwrap(),
        units("Terminal input")
    );
}
#[test]
fn cell_mapping_counts_utf16_units_and_normalizes_right_edge_once() {
    let columns = [0, 1, 1, 3, 4];
    for (offset, column) in [0.0, 1.0, 1.0, 3.0].into_iter().enumerate() {
        assert_eq!(
            map_point(10.0, offset, &columns, 4),
            Some(CellPoint { row: 10.0, column })
        );
    }
    assert_eq!(
        map_point(10.0, 4, &columns, 4),
        Some(CellPoint {
            row: 11.0,
            column: 0.0
        })
    );
    assert_eq!(
        map_point(10.0, usize::MAX, &columns, 8),
        Some(CellPoint {
            row: 10.0,
            column: 5.0
        })
    );
    assert!(map_point(f64::NAN, 0, &columns, 4).is_none());
    assert!(map_point(0.0, 0, &[], 4).unwrap().column.is_nan());
}
#[test]
fn invalid_range_errors_and_multiline_cell_count_are_exact() {
    let start = CellPoint {
        row: 3.0,
        column: 2.0,
    };
    for end in [
        start,
        CellPoint {
            row: 3.0,
            column: 1.0,
        },
        CellPoint {
            row: 2.0,
            column: 3.0,
        },
    ] {
        assert_eq!(
            selection_range(start, end, 8).unwrap_err().to_string(),
            "invalid range"
        );
    }
    let range = selection_range(
        start,
        CellPoint {
            row: 5.0,
            column: 1.0,
        },
        8,
    )
    .unwrap();
    assert_eq!((range.row, range.column, range.length), (3.0, 2.0, 15.0));
}
