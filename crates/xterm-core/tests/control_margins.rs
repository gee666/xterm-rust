// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/terminal_snapshot.rs"]
mod projection;
use projection::{events, parse_hex, snapshot};
use xterm_core::terminal::CoreTerminal;

#[test]
fn backspace_and_reverse_index_match_source_restriction_boundaries() {
    let mut failures = Vec::new();
    let mut count = 0;
    for (index, record) in include_str!("fixtures/control-margins.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = record.split('\t').collect();
        assert_eq!(fields.len(), 8);
        let mut terminal = CoreTerminal::new(5, 5, 3).unwrap();
        let mut setup = String::new();
        if fields[0] == "1" {
            setup.push_str("\x1b[?1049h");
        }
        setup.push_str("AAAAA\r\nBBBBB\r\nCCCCC\r\nDDDDD\r\nEEEEE\x1b[2;4r");
        if fields[1] == "1" {
            setup.push_str("\x1b[?6h");
        }
        if fields[2] == "1" {
            setup.push_str("\x1b[?45h");
        }
        terminal.write(&setup).unwrap();
        {
            let active = terminal.state().active_buffer();
            let mut buffer = active.borrow_mut();
            for row in 1..5 {
                buffer.set_wrapped(row, true).unwrap();
            }
            buffer.x = fields[3].parse().unwrap();
            buffer.y = fields[4].parse().unwrap();
        }
        events(&mut terminal);
        let result = terminal.write_utf16(&parse_hex(fields[5]));
        let actual = snapshot(&terminal);
        let notifications = events(&mut terminal);
        count += 1;
        if result.is_err() || actual != fields[6] || notifications != fields[7] {
            failures.push(format!("line {} alt/origin/reverse/x/y={:?}: error {:?}\nexpected {}\nactual {}\nevents expected {} actual {}\n", index + 1, &fields[..5], result.err(), fields[6], actual, fields[7], notifications));
        }
    }
    let output =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../results/control-margins");
    std::fs::create_dir_all(&output).unwrap();
    std::fs::write(output.join("counterexamples.txt"), failures.join("\n")).unwrap();
    std::fs::write(
        output.join("rust-summary.txt"),
        format!(
            "snapshots={count}\nfailed={}\npassed={}\n",
            failures.len(),
            count - failures.len()
        ),
    )
    .unwrap();
    for failure in failures.iter().take(2) {
        eprintln!("{failure}");
    }
    assert!(
        failures.is_empty(),
        "{} control restriction mismatches; see results/control-margins/counterexamples.txt",
        failures.len()
    );
}
