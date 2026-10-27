// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
// Ports BufferLine.test.ts constructor, edits, copying, resizing and combining.
#[path = "support/buffer_line.rs"]
mod support;
use support::{cell, utf16};
use xterm_core::buffer::{
    constants::{content, DEFAULT_ATTR},
    BufferLine, CellData, CharData,
};

fn array(line: &BufferLine) -> Vec<CharData> {
    (0..line.len()).map(|i| line.get(i).unwrap()).collect()
}
fn populated() -> BufferLine {
    let mut line = BufferLine::new(5, None, false);
    for (i, text) in ["a", "b", "c", "d", "e"].iter().enumerate() {
        line.set_cell(i, &cell((i + 1) as u32, text, 1));
    }
    line
}
fn expected(values: &[(u32, &str)]) -> Vec<CharData> {
    values
        .iter()
        .map(|&(a, s)| cell(a, s, 1).get_as_char_data())
        .collect()
}
#[test]
fn ctor() {
    let line = BufferLine::new(0, None, false);
    assert_eq!(line.len(), 0);
    assert!(!line.is_wrapped);
    for wrapped in [false, true] {
        let line = BufferLine::new(10, None, wrapped);
        assert_eq!(line.len(), 10);
        assert_eq!(line.get(0).unwrap(), cell(0, "", 1).get_as_char_data());
        assert_eq!(line.is_wrapped, wrapped);
    }
    let line = BufferLine::new(10, Some(&cell(123, "a", 456)), true);
    assert_eq!(line.len(), 10);
    assert_eq!(line.get(0).unwrap(), cell(123, "a", 456).get_as_char_data());
    assert!(line.is_wrapped);
}
#[test]
fn insert_cells() {
    let mut line = BufferLine::new(3, None, false);
    for (i, s) in ["a", "b", "c"].iter().enumerate() {
        line.set_cell(i, &cell(i as u32 + 1, s, 1));
    }
    line.insert_cells(1, 3, &cell(4, "d", 1));
    assert_eq!(array(&line), expected(&[(1, "a"), (4, "d"), (4, "d")]));
}
#[test]
fn delete_cells() {
    let mut line = populated();
    line.delete_cells(1, 2, &cell(6, "f", 1));
    assert_eq!(
        array(&line),
        expected(&[(1, "a"), (4, "d"), (5, "e"), (6, "f"), (6, "f")])
    );
}
#[test]
fn replace_cells() {
    let mut line = populated();
    line.replace_cells(2, 4, &cell(6, "f", 1), false);
    assert_eq!(
        array(&line),
        expected(&[(1, "a"), (2, "b"), (6, "f"), (6, "f"), (5, "e")])
    );
}
#[test]
fn fill() {
    let mut line = populated();
    line.fill(&cell(123, "z", 1), false);
    assert_eq!(array(&line), vec![cell(123, "z", 1).get_as_char_data(); 5]);
}
#[test]
fn clone() {
    let mut line = populated();
    line.is_wrapped = true;
    let copy = line.clone();
    assert_eq!(array(&copy), array(&line));
    assert_eq!(copy.len(), line.len());
    assert_eq!(copy.is_wrapped, line.is_wrapped);
}
#[test]
fn copy_from() {
    let line = populated();
    let mut copy = BufferLine::new(5, Some(&cell(1, "a", 1)), true);
    copy.copy_from(&line, false);
    assert_eq!(array(&copy), array(&line));
    assert_eq!(copy.len(), line.len());
    assert_eq!(copy.is_wrapped, line.is_wrapped);
}
#[test]
fn should_support_combining_chars() {
    let line = BufferLine::new(2, Some(&cell(1, "e\u{301}", 1)), false);
    assert_eq!(
        array(&line),
        vec![cell(1, "e\u{301}", 1).get_as_char_data(); 2]
    );
    let mut copy = BufferLine::new(5, Some(&cell(1, "a", 1)), true);
    copy.copy_from(&line, false);
    assert_eq!(array(&copy), array(&line));
    assert_eq!(array(&line.clone()), array(&line));
}
#[test]
fn resize_enlarge_false() {
    enlarge();
}
#[test]
fn resize_enlarge_true() {
    enlarge();
}
fn enlarge() {
    // Both upstream cases pass isWrapped=false despite their distinct titles.
    let mut line = BufferLine::new(5, Some(&cell(1, "a", 1)), false);
    line.resize(10, &cell(1, "a", 1));
    assert_eq!(array(&line), vec![cell(1, "a", 1).get_as_char_data(); 10]);
}
#[test]
fn resize_shrink_true() {
    let mut line = BufferLine::new(10, Some(&cell(1, "a", 1)), false);
    line.resize(5, &cell(1, "a", 1));
    assert_eq!(array(&line), vec![cell(1, "a", 1).get_as_char_data(); 5]);
}
#[test]
fn resize_shrink_to_zero() {
    let mut line = BufferLine::new(10, Some(&cell(1, "a", 1)), false);
    line.resize(0, &cell(1, "a", 1));
    assert!(array(&line).is_empty());
}
#[test]
fn resize_removes_cut_combining_data() {
    let mut line = BufferLine::new(10, Some(&cell(1, "a", 1)), false);
    let emoji = CharData {
        attr: 0,
        chars: utf16("😁"),
        width: 1,
        code: Some(0xd83d),
    };
    line.set(2, &emoji);
    line.set(9, &emoji);
    assert_eq!(
        line.translate_to_string(false, None, None, None),
        utf16("aa😁aaaaaa😁")
    );
    line.resize(5, &cell(1, "a", 1));
    assert_eq!(
        line.translate_to_string(false, None, None, None),
        utf16("aa😁aa")
    );
    line.resize(10, &cell(1, "a", 1));
    assert_eq!(
        line.translate_to_string(false, None, None, None),
        utf16("aa😁aaaaaaa")
    );
}
#[test]
fn trim_length_empty() {
    assert_eq!(
        BufferLine::new(10, Some(&cell(DEFAULT_ATTR, "", 1)), false).get_trimmed_length(),
        0
    );
}
#[test]
fn trim_length_ascii() {
    trim_length("a", 1, 3);
}
#[test]
fn trim_length_surrogate() {
    trim_length("𝄞", 1, 3);
}
#[test]
fn trim_length_combining() {
    trim_length("e\u{301}", 1, 3);
}
#[test]
fn trim_length_fullwidth() {
    trim_length("１", 2, 4);
}
fn trim_length(text: &str, width: i32, expected: usize) {
    let mut line = BufferLine::new(10, Some(&cell(DEFAULT_ATTR, "", 1)), false);
    line.set_cell(0, &cell(1, "a", 1));
    line.set_cell(2, &cell(1, text, width));
    if width == 2 {
        line.set_cell(3, &cell(0, "", 0));
    }
    assert_eq!(line.get_trimmed_length(), expected);
}
#[test]
fn combining_sets_width_on_empty_cell() {
    let mut line = BufferLine::new(3, Some(&cell(DEFAULT_ATTR, "", 1)), false);
    line.add_codepoint_to_cell(0, 0x301, 0);
    let actual = line.get_cell(0).unwrap();
    assert_eq!(
        actual.get_as_char_data(),
        cell(DEFAULT_ATTR, "\u{301}", 1).get_as_char_data()
    );
    assert_eq!(actual.is_combined(), 0);
}
#[test]
fn combining_add_to_combined_cell() {
    combining("e\u{301}", "e\u{301}\u{301}");
}
#[test]
fn combining_create_on_taken_cell() {
    combining("e", "e\u{301}");
}
fn combining(before: &str, after: &str) {
    let mut line = BufferLine::new(3, None, false);
    let mut work = CellData::default();
    line.load_cell(0, &mut work).unwrap();
    work.set_from_char_data(&cell(123, before, 1).get_as_char_data());
    line.set_cell(0, &work);
    line.add_codepoint_to_cell(0, 0x301, 0);
    line.load_cell(0, &mut work).unwrap();
    assert_eq!(
        work.get_as_char_data(),
        cell(123, after, 1).get_as_char_data()
    );
    assert_eq!(work.is_combined(), content::IS_COMBINED_MASK);
}
