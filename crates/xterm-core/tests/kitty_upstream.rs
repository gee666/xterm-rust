// Copyright (c) 2025 The xterm.js authors. All rights reserved.
// Licensed under MIT. Fixtures capture every unchanged upstream assertion input.
#[path = "support/kitty.rs"]
mod protocol;

#[test]
fn upstream_evaluation_assertions() {
    let fixture = include_str!("fixtures/kitty_upstream.tsv");
    assert_eq!(
        fixture.lines().count(),
        169,
        "upstream evaluation count must not shrink"
    );
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 11, "{line}");
        assert_eq!(
            protocol::evaluate_row(line).unwrap(),
            fields[7..10].join("\t"),
            "{}",
            fields[10]
        );
    }
}

#[test]
fn upstream_protocol_enablement_assertions() {
    let fixture = include_str!("fixtures/kitty_protocol_upstream.tsv");
    assert_eq!(
        fixture.lines().count(),
        4,
        "upstream protocol count must not shrink"
    );
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 3, "{line}");
        assert_eq!(
            xterm_core::input::kitty_keyboard::should_use_protocol(fields[0].parse().unwrap()),
            fields[1] == "1",
            "{}",
            fields[2]
        );
    }
}
