// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! Sparse slots need an explicit projection; the shared dense fixture unwraps rows.
use std::fmt::Write;
use xterm_core::{
    buffer::Buffer,
    terminal::{CoreTerminal, TerminalError, TerminalEvent},
};

fn hex(units: &[u16]) -> String {
    units
        .iter()
        .map(|u| format!("{u:x}"))
        .collect::<Vec<_>>()
        .join(",")
}
fn buffer_snapshot(b: &Buffer) -> String {
    let mut output = format!(
        "{},{},{},{},{},{}/",
        b.x, b.y, b.ybase, b.ydisp, b.scroll_top, b.scroll_bottom
    );
    for y in 0..b.lines.len() {
        if y > 0 {
            output.push('/');
        }
        let Some(row) = b.lines.get(y) else {
            output.push('-');
            continue;
        };
        let row = row.borrow();
        write!(output, "{}:", u8::from(row.is_wrapped)).unwrap();
        for x in 0..row.len() {
            if x > 0 {
                output.push(';');
            }
            let c = row.get_cell(x).unwrap();
            write!(
                output,
                "{},{},{},{},{},{}",
                c.content,
                c.fg,
                c.bg,
                hex(&c.get_chars()),
                c.extended.ext(),
                c.extended.url_id()
            )
            .unwrap();
        }
    }
    output
}
fn snapshot(t: &CoreTerminal) -> String {
    let s = t.state();
    let m = s.core.dec_private_modes.borrow();
    let a = &s.attributes;
    let normal = s.buffers.normal();
    let alt = s.buffers.alt();
    format!(
        "{},{},{},{},{},{},{},{},{}|{}|{}",
        s.cols(),
        s.rows(),
        u8::from(std::rc::Rc::ptr_eq(&s.active_buffer(), &alt)),
        a.fg,
        a.bg,
        a.extended.ext(),
        a.extended.url_id(),
        u8::from(m.origin),
        u8::from(m.reverse_wraparound),
        buffer_snapshot(&normal.borrow()),
        buffer_snapshot(&alt.borrow())
    )
}
fn notifications(t: &mut CoreTerminal) -> String {
    t.drain_events()
        .into_iter()
        .filter_map(|event| match event {
            TerminalEvent::Bell => Some("B".into()),
            TerminalEvent::CursorMove => Some("C".into()),
            TerminalEvent::Render { start, end } => Some(format!("R{start}:{end}")),
            // The oracle directly parses InputHandler, bypassing the JS host's
            // write queue. Its WriteParsed contract is tested by queue suites.
            TerminalEvent::WriteParsed => None,
            other => panic!("unexpected BS event {other:?}"),
        })
        .collect::<Vec<_>>()
        .join("/")
}

#[test]
fn backspace_sparse_rows_match_source_guards_partial_errors_and_events() {
    let fixture = include_str!("fixtures/backspace-sparse.tsv");
    assert_eq!(fixture.lines().count(), 480);
    let mut failures = Vec::new();
    let mut previous_errors = 0;
    for (index, row) in fixture.lines().enumerate() {
        let fields: Vec<_> = row.split('\t').collect();
        assert_eq!(fields.len(), 9);
        let mut t = CoreTerminal::new(5, 5, 3).unwrap();
        let setup = format!(
            "{}\x1b[31;44mAAAAA\r\nBBBBB\r\nCCCCC\r\nDDDDD\r\nEEEEE\r\nFFFFF\x1b[2;4r{}{}",
            if fields[0] == "1" { "\x1b[?1049h" } else { "" },
            if fields[1] == "1" { "\x1b[?6h" } else { "" },
            if fields[2] == "1" { "\x1b[?45h" } else { "" },
        );
        t.write(&setup).unwrap();
        let x: usize = fields[3].parse().unwrap();
        let y: usize = fields[4].parse().unwrap();
        {
            let active = t.state().active_buffer();
            let mut b = active.borrow_mut();
            let restricted_y = if fields[1] == "1" {
                y.clamp(1, 3)
            } else {
                y.min(4)
            };
            let current = b.ybase + restricted_y;
            b.set_wrapped(current, fields[5].starts_with("wrapped"))
                .unwrap();
            if fields[5] == "missing-current" {
                b.lines.set(current, None).unwrap();
            }
            if fields[5].ends_with("previous-missing") && current > 0 {
                b.lines.set(current - 1, None).unwrap();
            }
            b.x = x;
            b.y = y;
        }
        t.drain_events();
        let result = t.write("\x08\x07");
        let error_matches = if fields[6] == "1" {
            previous_errors += 1;
            matches!(result, Err(TerminalError::MissingLine))
        } else {
            result.is_ok()
        };
        let actual = snapshot(&t);
        let events = notifications(&mut t);
        if !error_matches || actual != fields[7] || events != fields[8] {
            failures.push(format!(
                "case {index} {:?}: {result:?}\nexpected {}\nactual {actual}\nevents expected {} actual {events}",
                &fields[..7], fields[7], fields[8]
            ));
        }
        t.dispose().unwrap();
    }
    assert!(
        previous_errors > 0,
        "previous-row errors must remain covered"
    );
    let output =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../results/backspace-sparse");
    std::fs::create_dir_all(&output).unwrap();
    std::fs::write(output.join("counterexamples.txt"), failures.join("\n\n")).unwrap();
    std::fs::write(
        output.join("native-summary.txt"),
        format!(
            "cases=480\npassed={}\nfailed={}\nexpected_previous_row_errors={previous_errors}\n",
            480 - failures.len(),
            failures.len()
        ),
    )
    .unwrap();
    assert!(
        failures.is_empty(),
        "{} sparse backspace mismatches; first={:?}",
        failures.len(),
        failures.first()
    );
}
