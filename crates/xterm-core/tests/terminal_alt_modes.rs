// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Original InputHandler alt-buffer/cursor assertions, including SGR cooperation.
use xterm_core::{buffer::AttributeData, terminal::CoreTerminal};
fn terminal() -> CoreTerminal {
    CoreTerminal::new(80, 30, 1000).unwrap()
}
fn line(t: &CoreTerminal, row: usize) -> String {
    let state = t.state();
    let buffer = state.active_buffer();
    let units = buffer
        .borrow()
        .translate_buffer_line_to_string(row, true, Some(0), None)
        .unwrap();
    String::from_utf16(&units).unwrap()
}
fn foreground(t: &CoreTerminal, row: usize, col: usize) -> AttributeData {
    t.state()
        .active_buffer()
        .borrow()
        .lines
        .get(row)
        .unwrap()
        .borrow()
        .get_cell(col)
        .unwrap()
        .attributes
}

#[test]
fn source_decset_decrst_47_retains_current_red_attribute() {
    let mut t = terminal();
    t.write("\x1b[?47h\r\n\x1b[31mJUNK\x1b[?47lTEST").unwrap();
    assert_eq!(line(&t, 0), "");
    assert_eq!(line(&t, 1), "    TEST");
    assert_eq!(foreground(&t, 1, 4).get_fg_color(), 1);
}
#[test]
fn source_decset_decrst_1047_retains_current_red_attribute() {
    let mut t = terminal();
    t.write("\x1b[?1047h\r\n\x1b[31mJUNK\x1b[?1047lTEST")
        .unwrap();
    assert_eq!(line(&t, 0), "");
    assert_eq!(line(&t, 1), "    TEST");
    assert_eq!(foreground(&t, 1, 4).get_fg_color(), 1);
}
#[test]
fn source_decset_decrst_1048_restores_cursor_and_default_attributes() {
    let mut t = terminal();
    t.write("\x1b[?1048h\r\n\x1b[31mJUNK\x1b[?1048lTEST")
        .unwrap();
    assert_eq!(line(&t, 0), "TEST");
    assert_eq!(line(&t, 1), "JUNK");
    assert_eq!(foreground(&t, 0, 0).fg, AttributeData::default().fg);
    assert_eq!(foreground(&t, 1, 0).get_fg_color(), 1);
}
#[test]
fn source_decset_decrst_1049_restores_cursor_and_default_attributes() {
    let mut t = terminal();
    t.write("\x1b[?1049h\r\n\x1b[31mJUNK\x1b[?1049lTEST")
        .unwrap();
    assert_eq!(line(&t, 0), "TEST");
    assert_eq!(line(&t, 1), "");
    assert_eq!(foreground(&t, 0, 0).fg, AttributeData::default().fg);
}
#[test]
fn source_decset_decrst_1049_retains_saved_cursor_for_alt_buffer() {
    let mut t = terminal();
    t.write("\x1b[?1049h\r\n\x1b[31m\x1b[s\x1b[?1049lTEST")
        .unwrap();
    assert_eq!(line(&t, 0), "TEST");
    assert_eq!(foreground(&t, 0, 0).fg, AttributeData::default().fg);
    t.write("\x1b[?1049h\x1b[uTEST").unwrap();
    assert_eq!(line(&t, 1), "TEST");
    assert_eq!(foreground(&t, 1, 0).get_fg_color(), 1);
}
#[test]
fn source_decset_decrst_1049_fills_alt_buffer_with_erase_background() {
    let mut t = terminal();
    t.write("\x1b[42m\x1b[?1049h").unwrap();
    assert_eq!(foreground(&t, 20, 10).get_bg_color(), 2);
}
