// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/terminal_osc_trace.rs"]
mod trace;

#[test]
fn full_osc_events_cells_buffers_and_link_lifetimes_match_original() {
    let mut runner = trace::Runner::new().unwrap();
    for (index, line) in include_str!("fixtures/terminal-osc-traces.tsv")
        .lines()
        .enumerate()
    {
        let (operation, expected) = line.split_once('\t').unwrap();
        assert_eq!(
            runner.step(operation).unwrap(),
            expected,
            "OSC operation{index}: {operation}"
        );
    }
}
