// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Faithful assertions from InputHandler.test.ts print, ISO-2022 and wide-cell suites.
#[path = "support/input_handler_core.rs"]
mod support;
use support::{cursor, lines, make};
use xterm_core::{
    charset::{builtin_charset, Charset},
    terminal::CoreTerminal,
};

#[test]
fn print_zero_width_regression_does_not_loop_or_add_rows() {
    let mut terminal = make(80, 30);
    let before = terminal.state().active_buffer().borrow().lines.len();
    terminal.write("\u{200b}").unwrap();
    assert_eq!(cursor(&terminal).1, 0);
    assert_eq!(
        terminal.state().active_buffer().borrow().lines.len(),
        before
    );
}
#[test]
fn print_joins_combining_characters_in_one_call() {
    let mut terminal = make(80, 30);
    terminal.write("e\u{301}").unwrap();
    assert_eq!(lines(&terminal, 1), ["e\u{301}"]);
    assert_eq!(cursor(&terminal).0, 1);
}
#[test]
fn print_joins_combining_characters_across_calls() {
    let mut terminal = make(80, 30);
    terminal.write("e").unwrap();
    terminal.write("\u{301}").unwrap();
    assert_eq!(lines(&terminal, 1), ["e\u{301}"]);
    assert_eq!(cursor(&terminal).0, 1);
}
#[test]
fn rep_repeats_the_preceding_grapheme_cluster() {
    let mut terminal = make(80, 30);
    terminal.write("e\u{301}\x1b[2b").unwrap();
    assert_eq!(lines(&terminal, 1), ["e\u{301}e\u{301}e\u{301}"]);
    assert_eq!(cursor(&terminal).0, 3);
}
#[test]
fn rep_without_preceding_join_state_does_not_print() {
    let mut terminal = make(80, 30);
    terminal.write("\x1b[2b").unwrap();
    assert_eq!(lines(&terminal, 1), [""]);
    assert_eq!(cursor(&terminal).0, 0);
}
#[test]
fn escape_sequence_resets_preceding_rep_state() {
    let mut terminal = make(80, 30);
    terminal.write("a\x1b[0m\x1b[2b").unwrap();
    assert_eq!(lines(&terminal, 1), ["a"]);
    assert_eq!(cursor(&terminal).0, 1);
}
#[test]
fn early_wrap_clears_cells_to_the_right() {
    let mut terminal = make(5, 5);
    terminal.write("12345").unwrap();
    terminal.state().active_buffer().borrow_mut().x = 0;
    terminal.write("￥￥￥").unwrap();
    assert_eq!(lines(&terminal, 2), ["￥￥", "￥"]);
}
#[test]
fn soft_hyphens_are_stripped() {
    let mut terminal = make(80, 30);
    terminal.write("Soft\u{ad}hy\u{ad}phen").unwrap();
    assert_eq!(lines(&terminal, 1), ["Softhyphen"]);
    assert_eq!(cursor(&terminal).0, 10);
}
#[test]
fn iso2022_g0_line_drawing() {
    let mut terminal = make(80, 30);
    terminal.write("\x1b(0q\x1b(Bq").unwrap();
    assert_eq!(lines(&terminal, 1), ["─q"]);
}
#[test]
fn iso2022_g1_line_drawing_after_shift_out() {
    let mut terminal = make(80, 30);
    terminal.write("\x1b)0\x0eq\x0f\x1b(Bq").unwrap();
    assert_eq!(lines(&terminal, 1), ["─q"]);
}
#[test]
fn iso2022_saved_cursor_restores_charset_and_glevel_identity() {
    let mut terminal = make(80, 30);
    let drawing = builtin_charset('0').unwrap();
    terminal.write("\x1b)0\x0e").unwrap();
    assert_eq!(terminal.state().charset.glevel, 1);
    assert!(Charset::ptr_eq(
        terminal.state().charset.charset.as_ref().unwrap(),
        &drawing
    ));
    terminal.write("\x1b7").unwrap();
    terminal.write("\x0f\x1b(B").unwrap();
    assert_eq!(terminal.state().charset.glevel, 0);
    assert!(terminal.state().charset.charset.is_none());
    terminal.write("\x1b8").unwrap();
    assert_eq!(terminal.state().charset.glevel, 1);
    assert!(Charset::ptr_eq(
        terminal.state().charset.charset.as_ref().unwrap(),
        &drawing
    ));
    terminal.write("q").unwrap();
    assert_eq!(lines(&terminal, 1), ["─"]);
}
fn wide() -> CoreTerminal {
    let mut terminal = make(10, 5);
    terminal.write(&"￥".repeat(20)).unwrap();
    terminal
}
#[test]
fn wide_cell_reset_by_print() {
    let mut terminal = wide();
    terminal.write("\x1b[H#").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["# ￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[1;6H######").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["# ￥ #####", "# ￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("#").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["# ￥ #####", "##￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("#").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["# ￥ #####", "### ￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[3;9H#").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["# ￥ #####", "### ￥￥￥", "￥￥￥￥#", "￥￥￥￥￥", ""]
    );
    terminal.write("#").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["# ￥ #####", "### ￥￥￥", "￥￥￥￥##", "￥￥￥￥￥", ""]
    );
    terminal.write("#").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["# ￥ #####", "### ￥￥￥", "￥￥￥￥##", "# ￥￥￥￥", ""]
    );
    terminal.write("\x1b[4;10H#").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["# ￥ #####", "### ￥￥￥", "￥￥￥￥##", "# ￥￥￥ #", ""]
    );
}
#[test]
fn wide_cell_reset_by_el() {
    let mut terminal = wide();
    terminal.write("\x1b[1;6H\x1b[K#").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥ #", "￥￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[2;5H\x1b[1K").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥ #", "      ￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[3;6H\x1b[1K").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥ #", "      ￥￥", "      ￥￥", "￥￥￥￥￥", ""]
    );
}
#[test]
fn wide_cell_reset_by_ich() {
    let mut terminal = wide();
    terminal.write("\x1b[1;6H\x1b[@").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥   ￥", "￥￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[2;4H\x1b[2@").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥   ￥", "￥    ￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[3;4H\x1b[3@").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥   ￥", "￥    ￥￥", "￥     ￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[4;4H\x1b[4@").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥   ￥", "￥    ￥￥", "￥     ￥", "￥      ￥", ""]
    );
}
#[test]
fn wide_cell_reset_by_dch() {
    let mut terminal = wide();
    terminal.write("\x1b[1;6H\x1b[P").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥ ￥￥", "￥￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[2;6H\x1b[2P").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥ ￥￥", "￥￥  ￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[3;6H\x1b[3P").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥ ￥￥", "￥￥  ￥", "￥￥ ￥", "￥￥￥￥￥", ""]
    );
}
#[test]
fn wide_cell_reset_by_ech() {
    let mut terminal = wide();
    terminal.write("\x1b[1;6H\x1b[X").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥  ￥￥", "￥￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[2;6H\x1b[2X").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥  ￥￥", "￥￥    ￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    terminal.write("\x1b[3;6H\x1b[3X").unwrap();
    assert_eq!(
        lines(&terminal, 5),
        ["￥￥  ￥￥", "￥￥    ￥", "￥￥    ￥", "￥￥￥￥￥", ""]
    );
}
