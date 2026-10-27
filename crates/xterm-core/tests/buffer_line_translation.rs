// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/buffer_line.rs"]
mod support;
use support::{cell, utf16};
use xterm_core::buffer::{constants::DEFAULT_ATTR, BufferLine};
fn blank() -> BufferLine {
    BufferLine::new(10, Some(&cell(DEFAULT_ATTR, "", 1)), false)
}
fn translate(
    line: &BufferLine,
    trim: bool,
    start: Option<usize>,
    end: Option<usize>,
    text: &str,
    columns: &[usize],
) {
    let mut actual_columns = vec![999];
    assert_eq!(
        line.translate_to_string(trim, start, end, Some(&mut actual_columns)),
        utf16(text)
    );
    assert_eq!(actual_columns, columns);
}
#[test]
fn empty_line() {
    let line = blank();
    translate(
        &line,
        false,
        None,
        None,
        "          ",
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    );
    translate(&line, true, None, None, "", &[0]);
}
#[test]
fn ascii() {
    let mut line = blank();
    for i in [0, 2, 4, 5] {
        line.set_cell(i, &cell(1, "a", 1));
    }
    translate(
        &line,
        false,
        None,
        None,
        "a a aa    ",
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    );
    translate(&line, true, None, None, "a a aa", &[0, 1, 2, 3, 4, 5, 6]);
    for trim in [true, false] {
        translate(&line, trim, Some(0), Some(5), "a a a", &[0, 1, 2, 3, 4, 5]);
        translate(&line, trim, Some(0), Some(4), "a a ", &[0, 1, 2, 3, 4]);
        translate(&line, trim, Some(0), Some(3), "a a", &[0, 1, 2, 3]);
    }
}
#[test]
fn surrogate() {
    multi_unit("𝄞");
}
#[test]
fn combining() {
    multi_unit("e\u{301}");
}
fn multi_unit(s: &str) {
    let mut line = blank();
    line.set_cell(0, &cell(1, "a", 1));
    for i in [2, 4, 5] {
        line.set_cell(i, &cell(1, s, 1));
    }
    translate(
        &line,
        false,
        None,
        None,
        &format!("a {s} {s}{s}    "),
        &[0, 1, 2, 2, 3, 4, 4, 5, 5, 6, 7, 8, 9, 10],
    );
    translate(
        &line,
        true,
        None,
        None,
        &format!("a {s} {s}{s}"),
        &[0, 1, 2, 2, 3, 4, 4, 5, 5, 6],
    );
    for trim in [true, false] {
        translate(
            &line,
            trim,
            Some(0),
            Some(5),
            &format!("a {s} {s}"),
            &[0, 1, 2, 2, 3, 4, 4, 5],
        );
        translate(
            &line,
            trim,
            Some(0),
            Some(4),
            &format!("a {s} "),
            &[0, 1, 2, 2, 3, 4],
        );
        translate(
            &line,
            trim,
            Some(0),
            Some(3),
            &format!("a {s}"),
            &[0, 1, 2, 2, 3],
        );
    }
}
#[test]
fn fullwidth() {
    let mut line = blank();
    line.set_cell(0, &cell(1, "a", 1));
    for i in [2, 5, 7] {
        line.set_cell(i, &cell(1, "１", 2));
        line.set_cell(i + 1, &cell(0, "", 0));
    }
    translate(
        &line,
        false,
        None,
        None,
        "a １ １１ ",
        &[0, 1, 2, 4, 5, 7, 9, 10],
    );
    translate(&line, true, None, None, "a １ １１", &[0, 1, 2, 4, 5, 7, 9]);
    for trim in [true, false] {
        for end in [7, 6] {
            translate(
                &line,
                trim,
                Some(0),
                Some(end),
                "a １ １",
                &[0, 1, 2, 4, 5, 7],
            );
        }
        translate(&line, trim, Some(0), Some(5), "a １ ", &[0, 1, 2, 4, 5]);
        for end in [4, 3] {
            translate(&line, trim, Some(0), Some(end), "a １", &[0, 1, 2, 4]);
        }
        translate(&line, trim, Some(0), Some(2), "a ", &[0, 1, 2]);
    }
}
#[test]
fn space_at_end() {
    let mut line = blank();
    for i in [0, 2, 4, 5] {
        line.set_cell(i, &cell(1, "a", 1));
    }
    line.set_cell(6, &cell(1, " ", 1));
    translate(
        &line,
        false,
        None,
        None,
        "a a aa    ",
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    );
    translate(
        &line,
        true,
        None,
        None,
        "a a aa ",
        &[0, 1, 2, 3, 4, 5, 6, 7],
    );
}
#[test]
fn should_always_return_sane_value() {
    let line = blank();
    translate(
        &line,
        false,
        None,
        None,
        "          ",
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
    );
    translate(&line, true, None, None, "", &[0]);
}
#[test]
fn end_column_zero() {
    let mut line = blank();
    line.set_cell(0, &cell(1, "a", 1));
    translate(&line, true, Some(0), Some(0), "", &[0]);
}
fn wide() -> BufferLine {
    let mut line = blank();
    for i in (0..10).step_by(2) {
        line.set_cell(i, &cell(1, "￥", 2));
    }
    line
}
fn text(line: &BufferLine, expected: &str) {
    assert_eq!(
        line.translate_to_string(false, None, None, None),
        utf16(expected)
    );
}
#[test]
fn insert_wide_char_at_pos() {
    let mut line = wide();
    let a = cell(1, "a", 1);
    line.insert_cells(9, 1, &a);
    text(&line, "￥￥￥￥ a");
    line.insert_cells(8, 1, &a);
    text(&line, "￥￥￥￥a ");
    line.insert_cells(1, 1, &a);
    text(&line, " a ￥￥￥a");
}
#[test]
fn insert_wide_char_at_end() {
    let mut line = wide();
    let a = cell(1, "a", 1);
    line.insert_cells(0, 3, &a);
    text(&line, "aaa￥￥￥ ");
    line.insert_cells(4, 1, &a);
    text(&line, "aaa a ￥￥");
    line.insert_cells(4, 1, &a);
    text(&line, "aaa aa ￥ ");
}
#[test]
fn delete_fullwidth() {
    let mut line = wide();
    let a = cell(1, "a", 1);
    line.delete_cells(0, 1, &a);
    text(&line, " ￥￥￥￥a");
    line.delete_cells(5, 2, &a);
    text(&line, " ￥￥￥aaa");
    line.delete_cells(0, 2, &a);
    text(&line, " ￥￥aaaaa");
}
#[test]
fn replace_fullwidth_start_zero() {
    for (end, expected) in [
        (1, "a ￥￥￥￥"),
        (2, "aa￥￥￥￥"),
        (3, "aaa ￥￥￥"),
        (8, "aaaaaaaa￥"),
        (9, "aaaaaaaaa "),
        (10, "aaaaaaaaaa"),
    ] {
        let mut line = wide();
        line.replace_cells(0, end, &cell(1, "a", 1), false);
        text(&line, expected);
    }
}
#[test]
fn replace_fullwidth_start_one() {
    for (end, expected) in [
        (2, " a￥￥￥￥"),
        (3, " aa ￥￥￥"),
        (4, " aaa￥￥￥"),
        (8, " aaaaaaa￥"),
        (9, " aaaaaaaa "),
        (10, " aaaaaaaaa"),
    ] {
        let mut line = wide();
        line.replace_cells(1, end, &cell(1, "a", 1), false);
        text(&line, expected);
    }
}
