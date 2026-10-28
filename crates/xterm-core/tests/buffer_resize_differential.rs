// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// All resize calls observed while executing the intact upstream Buffer.test.ts.
#[path = "support/buffer_snapshot.rs"]
mod snapshot;
use std::{cell::RefCell, rc::Rc};

#[test]
fn original_and_seeded_resizes_preserve_state_cells_identity_markers_and_events() {
    let cases = snapshot::cases(include_str!("fixtures/buffer-resize.tsv"));
    assert_eq!(
        cases.len(),
        1111,
        "86 original calls, 128 traces of 8 seeded calls and one directed regression"
    );
    assert_eq!(
        cases
            .iter()
            .filter(|c| c.title.starts_with("seed "))
            .count(),
        1024
    );
    for case in cases {
        let label = format!(
            "case {}: {} -> {}x{}",
            case.id, case.title, case.cols, case.rows
        );
        let (mut buffer, identities) = snapshot::hydrate(&case);
        snapshot::assert_snapshot(&buffer, &identities, &case.before, &label);
        let markers = buffer.markers.borrow().clone();
        let events = Rc::new(RefCell::new(Vec::new()));
        let mut subscriptions = Vec::new();
        let output = events.clone();
        subscriptions.push(
            buffer
                .lines
                .on_insert_emitter
                .event()
                .subscribe(move |event| {
                    output
                        .borrow_mut()
                        .push(format!("insert,{},{}", event.index, event.amount));
                }),
        );
        let output = events.clone();
        subscriptions.push(
            buffer
                .lines
                .on_delete_emitter
                .event()
                .subscribe(move |event| {
                    output
                        .borrow_mut()
                        .push(format!("delete,{},{}", event.index, event.amount));
                }),
        );
        let output = events.clone();
        subscriptions.push(
            buffer
                .lines
                .on_trim_emitter
                .event()
                .subscribe(move |amount| {
                    output.borrow_mut().push(format!("trim,{amount}"));
                }),
        );
        for (index, marker) in markers.iter().enumerate() {
            let output = events.clone();
            subscriptions.push(marker.on_dispose().subscribe(move |_| {
                output.borrow_mut().push(format!("marker,{index}"));
            }));
        }
        buffer
            .resize(case.cols, case.rows)
            .unwrap_or_else(|error| panic!("{label}: {error}"));
        snapshot::assert_snapshot(&buffer, &identities, &case.after, &label);
        let actual_markers: Vec<_> = markers
            .iter()
            .map(|m| (m.line_number(), m.is_disposed()))
            .collect();
        let expected_markers: Vec<_> = case
            .after
            .markers
            .iter()
            .map(|&(line, disposed)| (line as f64, disposed))
            .collect();
        assert_eq!(actual_markers, expected_markers, "{label}: markers");
        assert_eq!(
            buffer.markers.borrow().len(),
            case.after
                .markers
                .iter()
                .filter(|(_, disposed)| !disposed)
                .count(),
            "{label}: live markers"
        );
        let actual_events = if events.borrow().is_empty() {
            "-".into()
        } else {
            events.borrow().join("|")
        };
        assert_eq!(actual_events, case.events, "{label}: events");
        for subscription in subscriptions {
            subscription.dispose();
        }
    }
}
