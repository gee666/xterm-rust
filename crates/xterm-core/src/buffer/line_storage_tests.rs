// Copyright (c) 2018 The xterm.js authors. All rights reserved. MIT license.
use super::*;

fn cell(text: &str) -> CellData {
    CellData::from_char_data(&CharData {
        attr: 1,
        chars: text.encode_utf16().collect(),
        width: 1,
        code: None,
    })
}
#[test]
fn upstream_shrink_regrow_sparse_map_cardinality() {
    let mut line = BufferLine::new(10, Some(&cell("a")), false);
    let legacy = cell("😁").get_as_char_data();
    line.set(2, &legacy);
    line.set(9, &legacy);
    assert_eq!(line.combined.len(), 2);
    line.resize(5, &cell("a"));
    line.resize(10, &cell("a"));
    assert_eq!(line.combined.len(), 1);
}
#[test]
fn shrink_regrow_prunes_extended_identity() {
    let mut fill = cell("a");
    fill.bg |= bg_flags::HAS_EXTENDED;
    fill.extended.set_url_id(17);
    let mut line = BufferLine::new(10, Some(&fill), true);
    assert!(line.resize(4, &cell("b")));
    assert_eq!(line.extended.len(), 4);
    assert!(line.resize(4, &cell("b")));
    assert!(line.cleanup_memory());
    assert!(!line.cleanup_memory());
    assert!(!line.resize(10, &cell("b")));
    assert_eq!(line.get_extended(4).unwrap().url_id(), 0);
    assert_eq!(line.get_extended(3).unwrap().url_id(), 17);
    assert!(line.is_wrapped);
}
#[test]
fn cleanup_threshold_is_strict() {
    let mut line = BufferLine::new(10, None, false);
    assert!(!line.resize(5, &cell("")));
    assert!(!line.cleanup_memory());
    assert!(line.resize(4, &cell("")));
    assert!(line.cleanup_memory());
    assert_eq!(line.len(), 4);
    assert!(!line.resize(4, &cell("")));
}
#[test]
fn empty_combined_string_stays_empty() {
    let fill = CellData {
        content: content::IS_COMBINED_MASK | (1 << content::WIDTH_SHIFT),
        ..CellData::default()
    };
    let line = BufferLine::new(1, Some(&fill), false);
    let mut columns = vec![];
    assert!(line
        .translate_to_string(false, None, None, Some(&mut columns))
        .is_empty());
    assert_eq!(columns, vec![1]);
    assert_eq!(line.get(0).unwrap().code, None);
}
#[test]
fn blank_optimization_only_applies_to_blank_lines() {
    let blank = BufferLine::new(4, None, true);
    let cloned = blank.clone_line(true);
    assert_eq!(
        cloned.translate_to_string(false, None, None, None),
        vec![32; 4]
    );
    assert!(cloned.is_wrapped);
    assert!(cloned.combined.is_empty());
    assert!(cloned.extended.is_empty());
    let mut copied = BufferLine::new(2, Some(&cell("x")), false);
    copied.copy_from(&blank, true);
    assert_eq!(copied.translate_to_string(true, None, None, None), vec![]);
}
#[test]
fn empty_line_edits_and_missing_cells_do_not_panic() {
    let mut line = BufferLine::new(0, None, false);
    let fill = cell("x");
    line.insert_cells(usize::MAX, usize::MAX, &fill);
    line.delete_cells(usize::MAX, usize::MAX, &fill);
    line.replace_cells(usize::MAX, usize::MAX, &fill, true);
    line.set_cell(usize::MAX, &fill);
    line.set_cell_from_codepoint(usize::MAX, 0x61, 1, &fill);
    line.add_codepoint_to_cell(usize::MAX, 0x301, 0);
    assert!(line.get_cell(0).is_none());
    assert!(line.get_cell(usize::MAX).is_none());
    assert_eq!(line.translate_to_string(false, None, None, None), vec![]);
}
