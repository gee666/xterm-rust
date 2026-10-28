// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Both original reverse-wrap suites, including the earlier margin regression.
#[path = "support/input_handler_core.rs"]
mod support;
use support::{cursor, lines, make};
const TTY_BS: &str = "\x08 \x08";

#[test]
fn reverse_wrap_never_moves_outside_source_scroll_margins() {
    let mut terminal = make(5, 5);
    terminal.write("#####abcdefghijklmnopqrstuvwxy").unwrap();
    assert_eq!(
        lines(&terminal, 6),
        ["#####", "abcde", "fghij", "klmno", "pqrst", "uvwxy"]
    );
    assert_eq!(terminal.state().active_buffer().borrow().ydisp, 1);
    assert_eq!(cursor(&terminal).0, 5);
    assert_eq!(cursor(&terminal).1, 4);
    terminal.write(&TTY_BS.repeat(100)).unwrap();
    assert_eq!(
        lines(&terminal, 6),
        ["#####", "abcde", "fghij", "klmno", "pqrst", "    y"]
    );
    terminal.write("\x1b[?45h").unwrap();
    terminal.write("uvwxy").unwrap();
    terminal.write("\x1b[2;4r").unwrap();
    {
        let active = terminal.state().active_buffer();
        let mut buffer = active.borrow_mut();
        buffer.x = 5;
        buffer.y = 4;
    }
    terminal.write(&TTY_BS.repeat(100)).unwrap();
    assert_eq!(
        lines(&terminal, 6),
        ["#####", "abcde", "fghij", "klmno", "pqrst", "     "]
    );
    terminal.write("uvwxy").unwrap();
    {
        let active = terminal.state().active_buffer();
        let mut buffer = active.borrow_mut();
        buffer.x = 5;
        buffer.y = 3;
    }
    terminal.write(&TTY_BS.repeat(100)).unwrap();
    assert_eq!(
        lines(&terminal, 6),
        ["#####", "abcde", "     ", "     ", "     ", "uvwxy"]
    );
    assert_eq!(cursor(&terminal).0, 0);
    assert_eq!(
        cursor(&terminal).1,
        terminal.state().active_buffer().borrow().scroll_top
    );
    terminal.write("fghijklmnopqrst").unwrap();
    {
        let active = terminal.state().active_buffer();
        let mut buffer = active.borrow_mut();
        buffer.x = 5;
        buffer.y = 0;
    }
    terminal.write(&TTY_BS.repeat(100)).unwrap();
    assert_eq!(
        lines(&terminal, 6),
        ["#####", "     ", "fghij", "klmno", "pqrst", "uvwxy"]
    );
}
#[test]
fn default_backspace_cannot_delete_last_cell() {
    let mut terminal = make(5, 5);
    terminal.write("12345").unwrap();
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 1), ["123 5"]);
    terminal.write(&TTY_BS.repeat(10)).unwrap();
    assert_eq!(lines(&terminal, 1), ["    5"]);
}
#[test]
fn default_backspace_cannot_access_previous_line() {
    let mut terminal = make(5, 5);
    terminal.write(&"12345".repeat(2)).unwrap();
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 2), ["12345", "123 5"]);
    terminal.write(&TTY_BS.repeat(10)).unwrap();
    assert_eq!(lines(&terminal, 2), ["12345", "    5"]);
}
#[test]
fn reverse_wrap_can_delete_last_cell() {
    let mut terminal = make(5, 5);
    terminal.write("\x1b[?45h").unwrap();
    terminal.write("12345").unwrap();
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 1), ["1234 "]);
    terminal.write(&TTY_BS.repeat(7)).unwrap();
    assert_eq!(lines(&terminal, 1), ["     "]);
}
#[test]
fn reverse_wrap_can_access_a_wrapped_previous_line() {
    let mut terminal = make(5, 5);
    terminal.write("\x1b[?45h").unwrap();
    terminal.write(&"12345".repeat(2)).unwrap();
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 2), ["12345", "1234 "]);
    terminal.write(&TTY_BS.repeat(7)).unwrap();
    assert_eq!(lines(&terminal, 2), ["12   ", "     "]);
}
#[test]
fn reverse_wrap_lifts_the_wrapped_flag() {
    let mut terminal = make(5, 5);
    terminal.write("\x1b[?45h").unwrap();
    terminal.write(&"12345".repeat(2)).unwrap();
    assert!(
        terminal
            .state()
            .active_buffer()
            .borrow()
            .lines
            .get(1)
            .unwrap()
            .borrow()
            .is_wrapped
    );
    terminal.write(&TTY_BS.repeat(7)).unwrap();
    assert!(
        !terminal
            .state()
            .active_buffer()
            .borrow()
            .lines
            .get(1)
            .unwrap()
            .borrow()
            .is_wrapped
    );
}
#[test]
fn reverse_wrap_stops_at_hard_newlines() {
    let mut terminal = make(5, 5);
    terminal.write("\x1b[?45h").unwrap();
    terminal.write("12345\r\n").unwrap();
    terminal.write(&"12345".repeat(2)).unwrap();
    terminal.write(&TTY_BS.repeat(50)).unwrap();
    assert_eq!(lines(&terminal, 3), ["12345", "     ", "     "]);
    assert_eq!(cursor(&terminal).0, 0);
    assert_eq!(cursor(&terminal).1, 1);
}
#[test]
fn reverse_wrap_handles_wide_characters_and_early_wrap_padding() {
    let mut terminal = make(5, 5);
    terminal.write("\x1b[?45h").unwrap();
    terminal.write("￥￥￥").unwrap();
    assert_eq!(lines(&terminal, 2), ["￥￥", "￥"]);
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 2), ["￥￥", "  "]);
    assert_eq!(cursor(&terminal).0, 1);
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 2), ["￥￥", "  "]);
    assert_eq!(cursor(&terminal).0, 0);
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 2), ["￥  ", "  "]);
    assert_eq!(cursor(&terminal).0, 3);
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 2), ["￥  ", "  "]);
    assert_eq!(cursor(&terminal).0, 2);
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 2), ["    ", "  "]);
    assert_eq!(cursor(&terminal).0, 1);
    terminal.write(TTY_BS).unwrap();
    assert_eq!(lines(&terminal, 2), ["    ", "  "]);
    assert_eq!(cursor(&terminal).0, 0);
}
