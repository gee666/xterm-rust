// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::Cell, fmt::Write, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, BufferService},
    input::move_to_cell::move_to_cell_sequence,
    options::{OptionValue, OptionsService},
};
fn service(cols: usize, rows: usize, alt: bool) -> BufferService {
    service_with_scrollback(cols, rows, alt, 100)
}
fn service_with_scrollback(
    cols: usize,
    rows: usize,
    alt: bool,
    scrollback: usize,
) -> BufferService {
    let options = [("cols", cols), ("rows", rows), ("scrollback", scrollback)]
        .map(|(k, v)| (k, OptionValue::Number(v as f64)));
    let service = BufferService::new(Rc::new(OptionsService::new(false, options))).unwrap();
    if alt {
        service.buffers.activate_alt_buffer(None).unwrap();
    }
    service.buffer().borrow_mut().x = 3;
    service.buffer().borrow_mut().y = 3;
    service
}
#[test]
fn source_normal_directional_sequences() {
    let s = service(5, 5, false);
    for (x, expected) in [
        (1, "\x1b[D\x1b[D"),
        (2, "\x1b[D"),
        (4, "\x1b[C"),
        (5, "\x1b[C\x1b[C"),
    ] {
        assert_eq!(move_to_cell_sequence(x, 3, &s, false).unwrap(), expected);
    }
}
#[test]
fn source_normal_row_wrapping_sequences() {
    let s = service(5, 5, false);
    for y in [1, 2, 4, 5] {
        for x in 1..=5 {
            let distance: i64 = (y - 3) * 5 + x - 3;
            assert_eq!(
                move_to_cell_sequence(x, y, &s, false).unwrap(),
                if distance < 0 { "\x1b[D" } else { "\x1b[C" }
                    .repeat(distance.unsigned_abs() as usize)
            );
        }
    }
}
#[test]
fn source_application_cursor_sequences() {
    let s = service(5, 5, false);
    for (x, y, count, left) in [
        (3, 1, 10, true),
        (3, 2, 5, true),
        (2, 3, 1, true),
        (4, 3, 1, false),
        (3, 4, 5, false),
        (3, 5, 10, false),
    ] {
        assert_eq!(
            move_to_cell_sequence(x, y, &s, true).unwrap(),
            if left { "\x1bOD" } else { "\x1bOC" }.repeat(count)
        );
    }
}
#[test]
fn native_borrow_conflicts_are_errors_and_huge_normal_output_is_checked() {
    use xterm_core::input::move_to_cell::MoveToCellError;
    let service = service(5, 5, false);
    let buffer = service.buffer();
    let held = buffer.borrow_mut();
    assert!(matches!(
        move_to_cell_sequence(1, 1, &service, false),
        Err(MoveToCellError::Buffer(_))
    ));
    drop(held);
    // The source repeat helper returns empty for a negative cross-row count.
    assert_eq!(
        move_to_cell_sequence(i64::MAX, 1, &service, false).unwrap(),
        ""
    );
    assert!(matches!(
        move_to_cell_sequence(i64::MAX, 3, &service, false),
        Err(MoveToCellError::CoordinateRange) | Err(MoveToCellError::Allocation(_))
    ));
}
#[test]
fn normal_zero_scrollback_uses_the_unscrollable_branch() {
    let service = service_with_scrollback(5, 5, false, 0);
    assert!(!service.buffer().borrow().has_scrollback());
    assert_eq!(
        move_to_cell_sequence(4, 4, &service, false).unwrap(),
        "\x1b[B\x1b[C"
    );
}
#[test]
fn source_alt_cursor_across_rows() {
    let s = service(5, 5, true);
    assert_eq!(
        move_to_cell_sequence(4, 4, &s, false).unwrap(),
        "\x1b[B\x1b[C"
    );
}
fn snapshot(s: &BufferService) -> String {
    let b = s.buffer();
    let b = b.borrow();
    let mut result = format!("{},{}:{}:{}:{}", b.x, b.y, b.ybase, b.ydisp, b.lines.len());
    for i in 0..b.lines.len() {
        let line = b.lines.get(i).unwrap();
        let line = line.borrow();
        result.push_str(&format!("|{}", u8::from(line.is_wrapped)));
        for col in 0..line.len() {
            let chars = line
                .get_string(col)
                .unwrap()
                .iter()
                .map(|unit| format!("{unit:04x}"))
                .collect::<Vec<_>>()
                .join("-");
            result.push_str(&format!(
                ";{},{},{},{},{}",
                line.get_width(col).unwrap(),
                line.get_fg(col).unwrap(),
                line.get_bg(col).unwrap(),
                line.get_code_point(col).unwrap(),
                chars
            ));
        }
    }
    result
}
fn corpus(case_id: &str) {
    let mut cases = 0;
    let mut captures = String::new();
    for record in include_str!("fixtures/move-to-cell.tsv").lines() {
        let f: Vec<_> = record.split('\t').collect();
        if f[0] != case_id {
            continue;
        }
        let n = |i: usize| f[i].parse::<usize>().unwrap();
        let (cols, rows) = (n(1), n(2));
        let s = service(cols, rows, case_id == "move-02");
        let buffer = s.buffer();
        {
            let mut b = buffer.borrow_mut();
            b.x = n(3);
            b.y = n(4);
        }
        for row in 0..rows {
            let line = buffer.borrow().lines.get(row).unwrap();
            let mut line = line.borrow_mut();
            line.is_wrapped = f[8].as_bytes()[row] == b'1';
            for col in 0..cols {
                let pattern = (row + col + n(9)) % 5;
                let (code, width) = match pattern {
                    0 => (0x4e2d, 2),
                    1 => (0, 0),
                    2 => (0x1f600, 1),
                    3 => (0xd800, 1),
                    _ => (101, 1),
                };
                line.set_cell_from_codepoint(col, code, width, &AttributeData::default());
                if pattern == 4 {
                    line.add_codepoint_to_cell(col, 0x301, 0);
                }
            }
        }
        let events = Rc::new(Cell::new(0));
        let observed = events.clone();
        let _resize = s
            .on_resize()
            .subscribe(move |_| observed.set(observed.get() + 1));
        let observed = events.clone();
        let _scroll = s
            .on_scroll()
            .subscribe(move |_| observed.set(observed.get() + 1));
        let before = snapshot(&s);
        let actual = move_to_cell_sequence(
            f[5].parse().unwrap(),
            f[6].parse().unwrap(),
            &s,
            f[7] == "1",
        )
        .unwrap();
        let mut hex = String::with_capacity(actual.len() * 2);
        for byte in actual.as_bytes() {
            write!(hex, "{byte:02x}").unwrap();
        }
        let after = snapshot(&s);
        captures.push_str(&format!("{hex}\t{after}\t[]\n"));
        assert_eq!(hex, f[10], "case {cases}: {record}");
        assert_eq!(after, before);
        assert_eq!(events.get(), 0);
        assert!(Rc::ptr_eq(&buffer, &s.buffer()));
        cases += 1;
    }
    assert_eq!(cases, 512, "case corpus must contain exactly 512 rows");
    if let Some(path) = std::env::var_os("MOVE_CAPTURE") {
        std::fs::write(path, captures).unwrap();
    }
}
#[test]
fn move_01_differential_corpus() {
    corpus("move-01");
}
#[test]
fn move_02_differential_corpus() {
    corpus("move-02");
}
