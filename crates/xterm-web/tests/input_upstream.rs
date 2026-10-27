// Licensed under MIT. Captures every unchanged Clipboard/CompositionHelper test.
#[path = "support/input_trace.rs"]
mod protocol;

fn replay(fixture: &str, count: usize) {
    let mut runner = protocol::InputTrace::default();
    assert_eq!(
        fixture.lines().count(),
        count,
        "fixture step count must not shrink"
    );
    for line in fixture.lines() {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 6);
        assert_eq!(runner.apply(line).unwrap(), fields[4], "{}", fields[5]);
    }
}

#[test]
fn every_original_clipboard_and_ime_trace_step() {
    replay(include_str!("fixtures/browser-input-upstream.tsv"), 129);
}

#[test]
fn directed_deferred_and_utf16_regressions() {
    replay(include_str!("fixtures/browser-input-directed.tsv"), 72);
}
