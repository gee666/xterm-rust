// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Both wrappers execute real parser writes; exceptional source searches are retained.
#[path = "support/buffer_clear.rs"]
mod clear_callbacks;
#[path = "support/terminal_snapshot.rs"]
mod projection;
#[path = "support/buffer_terminal.rs"]
mod support;
use std::{fs, path::PathBuf};
use support::Harness;
use xterm_core::terminal::CoreTerminal;

fn compare(failures: &mut Vec<String>, label: &str, actual: &str, expected: &str) {
    if actual != expected {
        failures.push(format!("{label}\nexpected {expected}\nactual   {actual}\n"));
    }
}
#[test]
fn pa06_and_buffer_parser_addons_match_live_source_operations() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../results/buffer-terminal");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("rust-summary.json"),
        "{\"status\":\"incomplete\",\"overallEquivalence\":\"NOT PROVEN\"}\n",
    )
    .unwrap();
    let fixture = include_str!("fixtures/buffer-terminal.tsv");
    let mut current: Option<Harness> = None;
    let mut name = "";
    let mut failures = Vec::new();
    let mut snapshots = 0;
    let mut restores = 0;
    let mut traces = 0;
    for (index, row) in fixture.lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(
            fields.len(),
            19,
            "regenerate source fixture before exporting test: row {}",
            index + 1
        );
        if fields[0] != name {
            if let Some(previous) = current.take() {
                previous.finish().unwrap();
            }
            name = fields[0];
            traces += 1;
            current = Some(Harness::new(
                fields[1].parse().unwrap(),
                fields[2].parse().unwrap(),
                fields[3].parse().unwrap(),
            ));
        }
        let harness = current.as_mut().unwrap();
        let label = format!("trace {name} step {} operation {}", fields[4], fields[5]);
        harness.events.borrow_mut().clear();
        harness.marker_events.borrow_mut().clear();
        let mut marker_result = "-".to_owned();
        let clear_line = if fields[5] == "C" {
            harness.cursor_line()
        } else {
            None
        };
        let result = match fields[5] {
            "W" => harness
                .terminal
                .write_utf16(&projection::parse_hex(fields[6])),
            "R" => {
                let dimensions: Vec<_> = fields[6].split(',').map(|n| n.parse().unwrap()).collect();
                harness.terminal.resize(dimensions[0], dimensions[1])
            }
            "S" => harness.terminal.scroll_lines(fields[6].parse().unwrap()),
            "P" => harness.terminal.scroll_pages(fields[6].parse().unwrap()),
            "T" => harness.terminal.scroll_to_top(),
            "B" => harness.terminal.scroll_to_bottom(),
            "L" => harness
                .terminal
                .scroll_to_line_signed(fields[6].parse().unwrap()),
            "C" => harness.terminal.clear(),
            "Q" => harness.terminal.reset(),
            "M" | "A" | "D" => harness
                .add_marker(
                    if fields[5] == "D" {
                        0
                    } else {
                        fields[6].parse().unwrap()
                    },
                    fields[5] == "A",
                )
                .map(|ordinal| marker_result = ordinal),
            other => panic!("unknown fixture operation {other}"),
        };
        if let Err(error) = result {
            failures.push(format!("{label}: native operation failed: {error}"));
        }
        compare(
            &mut failures,
            &format!("{label}: full cells/buffers/modes"),
            &harness.state(),
            fields[7],
        );
        compare(
            &mut failures,
            &format!("{label}: saved cursor"),
            &harness.saved(),
            fields[8],
        );
        compare(
            &mut failures,
            &format!("{label}: marker creation"),
            &marker_result,
            fields[9],
        );
        compare(
            &mut failures,
            &format!("{label}: marker states"),
            &harness.marker_state(),
            fields[10],
        );
        let marker_events = if harness.marker_events.borrow().is_empty() {
            "-".into()
        } else {
            harness
                .marker_events
                .borrow()
                .iter()
                .map(usize::to_string)
                .collect::<Vec<_>>()
                .join(",")
        };
        compare(
            &mut failures,
            &format!("{label}: marker disposal order"),
            &marker_events,
            fields[11],
        );
        compare(
            &mut failures,
            &format!("{label}: ordered terminal events"),
            &harness.events.borrow().join("/"),
            fields[12],
        );
        let serialized = harness.serialized();
        let encoded = serialized
            .as_ref()
            .map_or_else(|error| format!("!{error}"), |text| projection::hex(text));
        compare(
            &mut failures,
            &format!("{label}: activated SerializeAddon output"),
            &encoded,
            fields[13],
        );
        compare(
            &mut failures,
            &format!("{label}: live SearchEngine outcomes"),
            &harness.search(),
            fields[14],
        );
        compare(
            &mut failures,
            &format!("{label}: public marker identities"),
            &harness.public_markers(),
            fields[16],
        );
        let clear_identity = if fields[5] == "C" {
            harness
                .terminal
                .with_state(|state| {
                    let active = state.active_buffer();
                    let first = active.borrow().lines.get(0);
                    let same = match (&clear_line, first) {
                        (Some(before), Some(after)) => std::rc::Rc::ptr_eq(before, &after),
                        (None, None) => true,
                        _ => false,
                    };
                    u8::from(same).to_string()
                })
                .unwrap()
        } else {
            "-".into()
        };
        compare(
            &mut failures,
            &format!("{label}: clear retains cursor row identity"),
            &clear_identity,
            fields[18],
        );
        if fields[15] != "-" {
            restores += 1;
            if let Ok(serialized) = serialized {
                let mut restored = CoreTerminal::new(
                    harness.terminal.cols().unwrap(),
                    harness.terminal.rows().unwrap(),
                    fields[3].parse().unwrap(),
                )
                .unwrap();
                if let Err(error) = restored.write_utf16(&serialized) {
                    failures.push(format!("{label}: native restore failed: {error}"));
                }
                compare(
                    &mut failures,
                    &format!("{label}: actual parser restore state"),
                    &projection::snapshot(&restored),
                    fields[15],
                );
                compare(
                    &mut failures,
                    &format!("{label}: restore events"),
                    &projection::events(&mut restored),
                    fields[17],
                );
            } else {
                failures.push(format!(
                    "{label}: no serialized data available for required restore"
                ));
            }
        }
        snapshots += 1;
    }
    if let Some(last) = current {
        last.finish().unwrap();
    }
    if (traces, snapshots) != (73, 862) {
        failures.push(format!("source corpus accounting changed: {traces} traces, {snapshots} snapshots; audit generator and mapping"));
    }
    fs::write(directory.join("rust-failures.txt"), failures.join("\n")).unwrap();
    fs::write(directory.join("rust-summary.json"), format!(
        "{{\"status\":\"{}\",\"platform\":\"{}\",\"traces\":{traces},\"snapshots\":{snapshots},\"restoreComparisons\":{restores},\"assertionFailures\":{},\"overallEquivalence\":\"NOT PROVEN\"}}\n",
        if failures.is_empty() { "passed" } else { "failed" }, std::env::consts::OS, failures.len())).unwrap();
    for failure in failures.iter().take(3) {
        eprintln!("{failure}");
    }
    assert!(
        failures.is_empty(),
        "{} mismatches retained in results/buffer-terminal/rust-failures.txt",
        failures.len()
    );
}

#[test]
fn pa06_marker_disposal_callbacks_read_and_reenter_public_terminal() {
    let directory = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../results/buffer-terminal");
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join("clear-rust-summary.json"),
        "{\"status\":\"incomplete\"}\n",
    )
    .unwrap();
    let mut failures = Vec::new();
    let mut cases = 0;
    for row in include_str!("fixtures/buffer-clear-reentry.tsv").lines() {
        let (action, expected) = row.split_once('\t').unwrap();
        let actual = clear_callbacks::trace(action);
        compare(
            &mut failures,
            &format!("clear callback {action}"),
            &actual,
            expected,
        );
        cases += 1;
    }
    if cases != 5 {
        failures.push(format!("expected five source callback cases, got {cases}"));
    }
    fs::write(
        directory.join("clear-rust-failures.txt"),
        failures.join("\n"),
    )
    .unwrap();
    fs::write(directory.join("clear-rust-summary.json"), format!(
        "{{\"status\":\"{}\",\"cases\":{cases},\"failures\":{},\"overallEquivalence\":\"NOT PROVEN\"}}\n",
        if failures.is_empty() { "passed" } else { "failed" }, failures.len())).unwrap();
    for failure in failures.iter().take(2) {
        eprintln!("{failure}");
    }
    assert!(
        failures.is_empty(),
        "clear callback mismatches retained in results/buffer-terminal/clear-rust-failures.txt"
    );
}
