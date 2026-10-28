// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
#[path="support/terminal_snapshot.rs"] mod projection;
use projection::{snapshot,events,parse_hex};
use xterm_core::terminal::CoreTerminal;
use std::path::PathBuf;

#[test]
fn both_buffers_cells_modes_and_ordered_events_match_real_source_traces() {
    let mut active = None;
    let mut name = "";
    let mut failures = Vec::new();
    let mut count = 0;
    for (index, line) in include_str!("fixtures/terminal-engine.tsv")
        .lines()
        .enumerate()
    {
        let fields: Vec<_> = line.split('\t').collect();
        assert_eq!(fields.len(), 8);
        if fields[0] != name {
            name = fields[0];
            active = Some(
                CoreTerminal::new(
                    fields[1].parse().unwrap(),
                    fields[2].parse().unwrap(),
                    fields[3].parse().unwrap(),
                )
                .unwrap(),
            );
        }
        let t = active.as_mut().unwrap();
        let result = match fields[4] {
            "W" => t.write_utf16(&parse_hex(fields[5])),
            "B" => t.write_bytes(
                &fields[5]
                    .split(',')
                    .map(|s| s.parse::<u8>().unwrap())
                    .collect::<Vec<_>>(),
            ),
            "R" => {
                let dims: Vec<usize> = fields[5].split(',').map(|s| s.parse().unwrap()).collect();
                t.resize(dims[0], dims[1])
            }
            "S" => t.scroll_lines(fields[5].parse().unwrap()),
            "C" => t.clear(),
            "Q" => t.reset(),
            other => panic!("unknown fixture action {other}"),
        };
        let actual = snapshot(t);
        let notifications = events(t);
        count += 1;
        if result.is_err() || actual != fields[6] || notifications != fields[7] {
            failures.push(format!("line {} case {} operation {} {} error {:?}\nexpected {}\nactual   {}\nevents expected {} actual {}\n",index+1,name,fields[4],fields[5],result.err(),fields[6],actual,fields[7],notifications));
        }
    }
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../results/terminal-engine");
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("rust-failures.txt"), failures.join("\n")).unwrap();
    std::fs::write(
        root.join("rust-summary.txt"),
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
    assert_eq!(
        failures.len(),
        0,
        "full-state differential failed; retained results/terminal-engine/rust-failures.txt"
    );
}
