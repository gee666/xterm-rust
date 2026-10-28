//! Exact UTF-16 comparisons against original serializer calls and seeded buffers.
#[path = "support/serialize_fixture.rs"]
mod fixture;

use std::fmt::Write;
use std::path::Path;
use xterm_core::serialize::{serialize_html, serialize_terminal, SerializeError, SerializeState};

fn hex(value: &[u16]) -> String {
    let mut output = String::new();
    for (index, unit) in value.iter().enumerate() {
        if index != 0 {
            output.push('.');
        }
        write!(output, "{unit:x}").unwrap();
    }
    output
}
fn result_text(value: &Result<Vec<u16>, String>) -> String {
    match value {
        Ok(value) => format!("ok:{}", hex(value)),
        Err(error) => format!("error:{error}"),
    }
}
fn run(kind: &str, expected_count: usize) {
    let cases = fixture::load();
    assert_eq!(cases.len(), 3350);
    let mut report = String::from("id\tstatus\tactual\texpected\n");
    let mut failed = Vec::new();
    let mut count = 0;
    for case in cases.iter().filter(|case| case.kind == kind) {
        count += 1;
        let actual: Result<Vec<u16>, SerializeError> = if kind == "ansi" {
            serialize_terminal(
                &SerializeState {
                    normal: &case.normal,
                    alternate: &case.alternate,
                    active_alternate: case.active_alternate,
                    cols: case.cols,
                    rows: case.rows,
                    current_attrs: &case.attrs,
                    modes: &case.modes,
                    scroll_top: case.margins[0],
                    scroll_bottom: case.margins[1],
                },
                &case.ansi,
            )
        } else {
            let buffer = if case.active_alternate {
                &case.alternate
            } else {
                &case.normal
            };
            serialize_html(
                buffer,
                case.cols,
                case.rows,
                case.selection,
                &case.config,
                &case.html,
            )
        };
        let actual = actual.map_err(|error| error.to_string());
        let passed = actual == case.expected;
        writeln!(
            report,
            "{}\t{}\t{}\t{}",
            case.id,
            if passed { "passed" } else { "failed" },
            result_text(&actual),
            result_text(&case.expected)
        )
        .unwrap();
        if !passed {
            failed.push(case.id);
        }
    }
    let results = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tmp/serialize");
    std::fs::create_dir_all(&results).unwrap();
    std::fs::write(results.join(format!("{kind}-differential.tsv")), report).unwrap();
    assert_eq!(count, expected_count);
    assert!(failed.is_empty(), "{kind}: {} of {count} cases differ. First IDs {:?}; inspect tmp/serialize/{kind}-differential.tsv and fixture inputs", failed.len(), &failed[..failed.len().min(20)]);
}

#[test]
fn ansi_matches_original_utf16_for_upstream_and_seeded_buffers() {
    run("ansi", 1663);
}

#[test]
fn html_matches_original_utf16_for_upstream_and_seeded_buffers() {
    run("html", 1687);
}
