// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/terminal_snapshot.rs"]
mod terminal_snapshot;
#[path = "support/input_handler_core_trace.rs"]
mod trace;

fn compare(accessibility: bool, expected_count: usize) {
    let rows = trace::replay().unwrap();
    assert_eq!(rows.len(), 221);
    let selected: Vec<_> = rows
        .iter()
        .enumerate()
        .filter(|(_, (expected, _))| (expected[4] == "1") == accessibility)
        .collect();
    assert_eq!(selected.len(), expected_count);
    let mismatches: Vec<_> = selected
        .into_iter()
        .filter(|(_, (expected, actual))| expected != actual)
        .map(|(index, (expected, actual))| (index, expected, actual))
        .collect();
    assert!(
        mismatches.is_empty(),
        "full-state source differences: {mismatches:#?}"
    );
}

#[test]
fn print_controls_modes_buffers_and_ordered_events_match_original() {
    compare(false, 219);
}
#[test]
fn accessibility_callbacks_observe_the_original_pre_cell_state() {
    compare(true, 2);
}
