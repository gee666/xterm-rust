// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// InputHandler.test.ts cursor positioning and DECSTBM source assertions.
use xterm_core::terminal::CoreTerminal;
fn terminal() -> CoreTerminal {
    CoreTerminal::new(10, 10, 1000).unwrap()
}
fn at(term: &CoreTerminal, x: usize, y: usize) {
    let active = term.state().active_buffer();
    let mut b = active.borrow_mut();
    b.x = x;
    b.y = y;
}
fn check(term: &mut CoreTerminal, text: &str, expected: (usize, usize)) {
    term.write(text).unwrap();
    let active = term.state().active_buffer();
    let b = active.borrow();
    assert_eq!((b.x, b.y), expected, "input {text:?}");
}
#[test]
fn cursor_forward_and_horizontal_relative() {
    for command in ['C', 'a'] {
        let mut t = terminal();
        for (param, x) in [("", 1), ("1", 2), ("4", 6), ("100", 9)] {
            check(&mut t, &format!("\x1b[{param}{command}"), (x, 0));
        }
        at(&t, 8, 4);
        check(&mut t, &format!("\x1b[{command}"), (9, 4));
    }
}
#[test]
fn cursor_backward() {
    let mut t = terminal();
    check(&mut t, "\x1b[D", (0, 0));
    check(&mut t, "\x1b[1D", (0, 0));
    t.write("\x1b[100C").unwrap();
    for (param, x) in [("", 8), ("1", 7), ("4", 3), ("100", 0)] {
        check(&mut t, &format!("\x1b[{param}D"), (x, 0));
    }
    at(&t, 4, 4);
    check(&mut t, "\x1b[D", (3, 4));
}
#[test]
fn cursor_down_and_vertical_relative() {
    for command in ['B', 'e'] {
        let mut t = terminal();
        for (param, y) in [("", 1), ("1", 2), ("4", 6), ("100", 9)] {
            check(&mut t, &format!("\x1b[{param}{command}"), (0, y));
        }
        let y = if command == 'B' { 0 } else { 4 };
        at(&t, 8, y);
        check(&mut t, &format!("\x1b[{command}"), (8, y + 1));
    }
}
#[test]
fn cursor_up() {
    let mut t = terminal();
    check(&mut t, "\x1b[A", (0, 0));
    check(&mut t, "\x1b[1A", (0, 0));
    t.write("\x1b[100B").unwrap();
    for (param, y) in [("", 8), ("1", 7), ("4", 3), ("100", 0)] {
        check(&mut t, &format!("\x1b[{param}A"), (0, y));
    }
    at(&t, 8, 9);
    check(&mut t, "\x1b[A", (8, 8));
}
#[test]
fn cursor_next_line() {
    let mut t = terminal();
    for (param, y) in [("", 1), ("1", 2), ("4", 6), ("100", 9)] {
        check(&mut t, &format!("\x1b[{param}E"), (0, y));
    }
    at(&t, 8, 0);
    check(&mut t, "\x1b[E", (0, 1));
}
#[test]
fn cursor_previous_line() {
    let mut t = terminal();
    check(&mut t, "\x1b[F", (0, 0));
    check(&mut t, "\x1b[1F", (0, 0));
    t.write("\x1b[100E").unwrap();
    for (param, y) in [("", 8), ("1", 7), ("4", 3), ("100", 0)] {
        check(&mut t, &format!("\x1b[{param}F"), (0, y));
    }
    at(&t, 8, 9);
    check(&mut t, "\x1b[F", (0, 8));
}
#[test]
fn horizontal_absolute_aliases() {
    for command in ['G', '`'] {
        let mut t = terminal();
        for (param, x) in [("", 0), ("1", 0), ("2", 1), ("5", 4), ("100", 9)] {
            check(&mut t, &format!("\x1b[{param}{command}"), (x, 0));
        }
    }
}
#[test]
fn vertical_absolute() {
    let mut t = terminal();
    for (param, y) in [("", 0), ("1", 0), ("2", 1), ("5", 4), ("100", 9)] {
        check(&mut t, &format!("\x1b[{param}d"), (0, y));
    }
    at(&t, 8, 4);
    check(&mut t, "\x1b[d", (8, 0));
}
#[test]
fn cursor_position() {
    let mut t = terminal();
    for (sequence, expected) in [
        ("H", (0, 0)),
        ("1H", (0, 0)),
        ("1;1H", (0, 0)),
        ("8H", (0, 7)),
        (";8H", (7, 0)),
        ("100;100H", (9, 9)),
    ] {
        at(&t, 5, 5);
        check(&mut t, &format!("\x1b[{sequence}"), expected);
    }
}
#[test]
fn cursor_position_origin_and_scroll_margins() {
    let mut t = terminal();
    check(&mut t, "\x1b[?6h\x1b[2;3r\x1b[1;1H", (0, 1));
    t.write("X").unwrap();
    let b = t.state().active_buffer();
    assert_eq!(
        b.borrow()
            .translate_buffer_line_to_string(1, true, None, None)
            .unwrap(),
        vec![88]
    );
    check(&mut t, "\x1b[2;1H", (0, 2));
    check(&mut t, "\x1b[10;10H", (9, 2));
    t.write("\x1b[?6l").unwrap();
    check(&mut t, "\x1b[2;1H", (0, 1));
}
#[test]
fn clamp_positive_out_of_range_cursor() {
    for (sequence, expected) in [
        ("C", (9, 9)),
        ("D", (8, 9)),
        ("B", (9, 9)),
        ("A", (9, 8)),
        ("E", (0, 9)),
        ("F", (0, 8)),
        ("5G", (4, 9)),
        ("5;5H", (4, 4)),
        ("5`", (4, 9)),
        ("a", (9, 9)),
        ("5d", (9, 4)),
        ("e", (9, 9)),
        ("P", (9, 9)),
        ("X", (9, 9)),
        ("@", (9, 9)),
    ] {
        let mut t = terminal();
        at(&t, 10000, 10000);
        check(&mut t, &format!("\x1b[{sequence}"), expected);
    }
}
#[test]
fn pending_wrap_edit_deletes_last_cell() {
    for command in ['P', 'X', '@'] {
        let mut t = terminal();
        t.write(&format!("0123456789\x1b[{command}")).unwrap();
        let active = t.state().active_buffer();
        let row = active.borrow().lines.get(0).unwrap();
        assert_eq!(
            row.borrow().translate_to_string(false, None, None, None),
            "012345678 ".encode_utf16().collect::<Vec<_>>()
        );
    }
}
fn margins(term: &CoreTerminal) -> (usize, usize) {
    let active = term.state().active_buffer();
    let b = active.borrow();
    (b.scroll_top, b.scroll_bottom)
}
#[test]
fn scroll_margin_defaults() {
    let mut t = terminal();
    for (sequence, expected) in [("r", (0, 9)), ("3;7r", (2, 6)), ("0;0r", (0, 9))] {
        t.write(&format!("\x1b[{sequence}")).unwrap();
        assert_eq!(margins(&t), expected);
    }
}
#[test]
fn scroll_margin_clamps_bottom() {
    let mut t = terminal();
    t.write("\x1b[3;1000r").unwrap();
    assert_eq!(margins(&t), (2, 9));
}
#[test]
fn scroll_margin_rejects_top_at_or_below_bottom() {
    let mut t = terminal();
    t.write("\x1b[7;2r").unwrap();
    assert_eq!(margins(&t), (0, 9));
}
#[test]
fn scroll_margin_homes_cursor() {
    let mut t = terminal();
    at(&t, 10000, 10000);
    check(&mut t, "\x1b[2;7r", (0, 0));
}
