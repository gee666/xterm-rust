// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
#[path = "support/buffer_line.rs"]
mod support;
use support::{cell, utf16};
use xterm_core::buffer::{
    constants::{bg_flags, underline_style},
    BufferLine, CellData, ExtendedAttrs,
};

fn extended(line: &BufferLine, index: usize) -> Option<ExtendedAttrs> {
    let cell = line.get_cell(index).unwrap();
    if cell.has_extended_attrs() != 0 {
        Some(cell.attributes.extended)
    } else {
        None
    }
}
fn style(line: &BufferLine, index: usize) -> Option<u32> {
    extended(line, index).map(|e| e.underline_style())
}
fn setup() -> BufferLine {
    let mut line = BufferLine::new(5, None, false);
    let mut a = cell(1, "a", 1);
    line.set_cell(0, &a);
    a.extended.set_underline_style(underline_style::CURLY);
    a.bg |= bg_flags::HAS_EXTENDED;
    line.set_cell(1, &a);
    // The upstream loadCell/clone/copyFrom cases use a raw content=65.
    a.content = 65;
    line.set_cell(2, &a);
    a.attributes.extended = a.extended.clone();
    a.extended.set_underline_style(underline_style::DOTTED);
    line.set_cell(3, &a);
    a.bg &= !bg_flags::HAS_EXTENDED;
    line.set_cell(4, &a);
    line
}
#[test]
fn set_cells() {
    let mut line = BufferLine::new(5, None, false);
    let mut a = cell(1, "a", 1);
    line.set_cell(0, &a);
    a.extended.set_underline_style(underline_style::CURLY);
    a.bg |= bg_flags::HAS_EXTENDED;
    line.set_cell(1, &a);
    a.content = cell(1, "A", 1).content;
    line.set_cell(2, &a);
    a.attributes.extended = a.extended.clone();
    a.extended.set_underline_style(underline_style::DOTTED);
    line.set_cell(3, &a);
    a.bg &= !bg_flags::HAS_EXTENDED;
    line.set_cell(4, &a);
    for (i, s) in ["a", "a", "A", "A", "A"].iter().enumerate() {
        assert_eq!(line.get(i).unwrap(), cell(1, s, 1).get_as_char_data());
    }
    assert_eq!(style(&line, 0), None);
    assert_eq!(style(&line, 1), Some(underline_style::CURLY));
    assert_eq!(style(&line, 2), Some(underline_style::CURLY));
    assert_eq!(style(&line, 3), Some(underline_style::DOTTED));
    assert_eq!(style(&line, 4), None);
    assert!(extended(&line, 1)
        .unwrap()
        .same_identity(&extended(&line, 2).unwrap()));
    assert!(!extended(&line, 1)
        .unwrap()
        .same_identity(&extended(&line, 3).unwrap()));
}
#[test]
fn load_cell() {
    let line = setup();
    let mut cells: Vec<_> = (0..5).map(|_| CellData::default()).collect();
    for (i, c) in cells.iter_mut().enumerate() {
        line.load_cell(i, c).unwrap();
    }
    for (c, style) in cells.iter().zip([
        underline_style::NONE,
        underline_style::CURLY,
        underline_style::CURLY,
        underline_style::DOTTED,
        underline_style::NONE,
    ]) {
        assert_eq!(c.extended.underline_style(), style);
    }
    assert!(cells[1].extended.same_identity(&cells[2].extended));
    assert!(!cells[2].extended.same_identity(&cells[3].extended));
}
fn curly() -> CellData {
    let mut a = cell(1, "a", 1);
    a.extended.set_underline_style(underline_style::CURLY);
    a.bg |= bg_flags::HAS_EXTENDED;
    a
}
#[test]
fn fill() {
    let mut line = BufferLine::new(3, None, false);
    line.fill(&curly(), false);
    for i in 0..3 {
        assert_eq!(style(&line, i), Some(underline_style::CURLY));
    }
}
#[test]
fn insert_cells() {
    let mut line = BufferLine::new(5, None, false);
    let mut a = curly();
    line.insert_cells(1, 3, &a);
    for i in 1..4 {
        assert_eq!(style(&line, i), Some(underline_style::CURLY));
    }
    assert_eq!(style(&line, 4), None);
    a.attributes.extended = a.extended.clone();
    a.extended.set_underline_style(underline_style::DOTTED);
    line.insert_cells(2, 2, &a);
    for (i, s) in [
        (1, underline_style::CURLY),
        (2, underline_style::DOTTED),
        (3, underline_style::DOTTED),
        (4, underline_style::CURLY),
    ] {
        assert_eq!(style(&line, i), Some(s));
    }
}
#[test]
fn delete_cells() {
    let mut line = BufferLine::new(5, None, false);
    let mut a = curly();
    line.fill(&a, false);
    a.attributes.extended = a.extended.clone();
    a.extended.set_underline_style(underline_style::DOUBLE);
    line.delete_cells(1, 3, &a);
    for i in 0..5 {
        assert_eq!(
            style(&line, i),
            Some(if i < 2 {
                underline_style::CURLY
            } else {
                underline_style::DOUBLE
            })
        );
    }
}
#[test]
fn replace_cells() {
    let mut line = BufferLine::new(5, None, false);
    let mut a = curly();
    line.fill(&a, false);
    a.attributes.extended = a.extended.clone();
    a.extended.set_underline_style(underline_style::DOUBLE);
    line.replace_cells(1, 3, &a, false);
    for i in 0..5 {
        assert_eq!(
            style(&line, i),
            Some(if (1..3).contains(&i) {
                underline_style::DOUBLE
            } else {
                underline_style::CURLY
            })
        );
    }
}
fn same_extended(a: &BufferLine, b: &BufferLine) {
    for i in 0..5 {
        match (extended(a, i), extended(b, i)) {
            (Some(x), Some(y)) => assert!(x.same_identity(&y)),
            (None, None) => (),
            _ => panic!("extended attr mismatch at {i}"),
        }
    }
}
#[test]
fn clone() {
    let line = setup();
    same_extended(&line.clone(), &line);
}
#[test]
fn copy_from() {
    let initial = setup();
    let mut line = BufferLine::new(5, None, false);
    line.fill(&cell(1, "b", 1), false);
    line.copy_from(&initial, false);
    same_extended(&line, &initial);
}
#[test]
fn reused_cell_does_not_reset_previous_extended_entry() {
    let mut line = BufferLine::new(5, None, false);
    line.set_cell(0, &curly());
    let mut work = CellData::default();
    line.load_cell(0, &mut work).unwrap();
    line.load_cell(1, &mut work).unwrap();
    assert_eq!(style(&line, 0), Some(underline_style::CURLY));
    line.insert_cells(1, 1, &cell(0, "", 1));
    assert_eq!(style(&line, 0), Some(underline_style::CURLY));
}
#[test]
fn protected_edits_preserve_flagged_cells_and_wide_boundaries() {
    let mut line = BufferLine::new(6, None, false);
    let mut wide = cell(1, "￥", 2);
    wide.bg |= bg_flags::PROTECTED;
    line.set_cell(0, &wide);
    line.set_cell(1, &cell(0, "", 0));
    line.set_cell(2, &cell(1, "￥", 2));
    let mut end = cell(0, "", 0);
    end.bg |= bg_flags::PROTECTED;
    line.set_cell(3, &end);
    line.replace_cells(1, 3, &cell(1, "a", 1), true);
    assert_eq!(
        line.translate_to_string(false, None, None, None),
        utf16("￥a   ")
    );
    assert_eq!(line.get_width(0), Some(2));
    assert_eq!(line.get_width(3), Some(0));
    line.fill(&cell(1, "b", 1), true);
    assert_eq!(line.get_string(0), Some(utf16("￥")));
    assert_eq!(line.get_width(3), Some(0));
}
