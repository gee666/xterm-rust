// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/terminal_snapshot.rs"]
mod projection;
use projection::{events, parse_hex, snapshot};
use xterm_core::terminal::CoreTerminal;

#[test]
fn every_csi_family_matches_real_terminal_cells_modes_and_events() {
    let mut terminal = None;
    let mut id = "";
    let mut failures = Vec::new();
    let mut count = 0;
    for (index, line) in include_str!("fixtures/csi-terminal.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 7);
        if fields[0] != id {
            id = fields[0];
            terminal = Some(
                CoreTerminal::new(
                    fields[1].parse().unwrap(),
                    fields[2].parse().unwrap(),
                    fields[3].parse().unwrap(),
                )
                .unwrap(),
            );
        }
        let t = terminal.as_mut().unwrap();
        let input = parse_hex(fields[4]);
        let result = t.write_utf16(&input);
        let actual = snapshot(t);
        let notifications = events(t);
        count += 1;
        if result.is_err() || actual != fields[5] || notifications != fields[6] {
            failures.push(format!("line {} trace {id} input {} error {:?}\nexpected {}\nactual {}\nevents expected {} actual {}\n",
                index + 1, fields[4], result.err(), fields[5], actual, fields[6], notifications));
        }
    }
    let output = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../results/csi");
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
    for failure in failures.iter().take(3) {
        eprintln!("{failure}");
    }
    assert!(
        failures.is_empty(),
        "CSI differential failures retained in results/csi/counterexamples.txt"
    );
}
