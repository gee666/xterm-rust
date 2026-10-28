// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
// All seven assertion cases in common/buffer/BufferReflow.test.ts.
#[path = "support/buffer_line.rs"]
mod support;
use std::{cell::RefCell, rc::Rc};
use support::{cell, utf16};
use xterm_core::{
    buffer::{reflow::*, BufferLine},
    circular_list::CircularList,
};

type Line = Rc<RefCell<BufferLine>>;
fn row(text: &str, cols: usize, wrapped: bool) -> Line {
    let mut line = BufferLine::new(cols, None, wrapped);
    let mut col = 0;
    for c in text.chars() {
        let width = if c == '汉' || c == '语' { 2 } else { 1 };
        line.set_cell(col, &cell(0, &c.to_string(), width));
        if width == 2 {
            line.set_cell(col + 1, &cell(0, "", 0));
        }
        col += width as usize;
    }
    Rc::new(RefCell::new(line))
}
fn text(line: &Line, trim: bool) -> Vec<u16> {
    line.borrow().translate_to_string(trim, None, None, None)
}
fn lengths(lines: &[Line], old: usize, new: usize) -> Vec<usize> {
    reflow_smaller_get_new_line_lengths(lines, old, new).unwrap()
}
fn wrapped(text: &str) -> CircularList<Line> {
    let lines = CircularList::new(text.len()).unwrap();
    for (i, c) in text.chars().enumerate() {
        lines.push(row(&c.to_string(), 1, i > 0)).unwrap();
    }
    lines
}
#[test]
fn small_wide_line() {
    let line = row("汉语", 4, false);
    assert_eq!(text(&line, true), utf16("汉语"));
    for new in [3, 2] {
        assert_eq!(lengths(std::slice::from_ref(&line), 4, new), [2, 2]);
    }
}
#[test]
fn large_wide_line() {
    let line = row("汉语汉语汉语", 12, false);
    assert_eq!(text(&line, false), utf16("汉语汉语汉语"));
    for (new, expected) in [
        (11, vec![10, 2]),
        (10, vec![10, 2]),
        (9, vec![8, 4]),
        (8, vec![8, 4]),
        (7, vec![6, 6]),
        (6, vec![6, 6]),
        (5, vec![4, 4, 4]),
        (4, vec![4, 4, 4]),
        (3, vec![2, 2, 2, 2, 2, 2]),
        (2, vec![2, 2, 2, 2, 2, 2]),
    ] {
        assert_eq!(lengths(std::slice::from_ref(&line), 12, new), expected);
    }
}
#[test]
fn mixed_wide_and_single() {
    let line = row("a汉语b", 6, false);
    assert_eq!(text(&line, false), utf16("a汉语b"));
    for (new, expected) in [
        (5, vec![5, 1]),
        (4, vec![3, 3]),
        (3, vec![3, 3]),
        (2, vec![1, 2, 2, 1]),
    ] {
        assert_eq!(lengths(std::slice::from_ref(&line), 6, new), expected);
    }
}
#[test]
fn wrapped_wide_and_single() {
    let lines = [row("a汉语b", 6, false), row("a汉语b", 6, true)];
    for line in &lines {
        assert_eq!(text(line, false), utf16("a汉语b"));
    }
    for (new, expected) in [
        (5, vec![5, 4, 3]),
        (4, vec![3, 4, 4, 1]),
        (3, vec![3, 3, 3, 3]),
        (2, vec![1, 2, 2, 2, 2, 2, 1]),
    ] {
        assert_eq!(lengths(&lines, 6, new), expected);
    }
}
#[test]
fn trailing_null_space() {
    let line = row("汉语", 5, false);
    assert_eq!(text(&line, true), utf16("汉语"));
    assert_eq!(text(&line, false), utf16("汉语 "));
    for new in [3, 2] {
        assert_eq!(lengths(std::slice::from_ref(&line), 4, new), [2, 2]);
    }
}
#[test]
fn cursor_block_skipped_unless_enabled() {
    let lines = wrapped("abcde");
    let null = cell(0, "", 1);
    let skipped = reflow_larger_get_lines_to_remove(&lines, 1, 5, 2, &null, false).unwrap();
    let reflowed = reflow_larger_get_lines_to_remove(&lines, 1, 5, 2, &null, true).unwrap();
    assert!(skipped.is_empty());
    assert!(!reflowed.is_empty());
}
#[test]
fn cursor_outside_wrapped_block() {
    let lines = wrapped("abcde");
    assert!(
        !reflow_larger_get_lines_to_remove(&lines, 1, 5, 10, &cell(0, "", 1), false)
            .unwrap()
            .is_empty()
    );
}
