// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
// InputHandler.test.ts SL/SR/DECIC/DECDC, scroll-margin and REP assertions.
use xterm_core::terminal::CoreTerminal;
fn lines(term: &CoreTerminal, limit: usize, trim: bool) -> Vec<Vec<u16>> {
    let active = term.state().active_buffer();
    let b = active.borrow();
    (0..limit)
        .filter_map(|i| b.lines.get(i))
        .map(|row| row.borrow().translate_to_string(trim, None, None, None))
        .collect()
}
fn check(term: &mut CoreTerminal, input: &str, expected: &[&str]) {
    term.write(input).unwrap();
    assert_eq!(
        lines(term, expected.len(), true),
        expected
            .iter()
            .map(|s| s.encode_utf16().collect::<Vec<_>>())
            .collect::<Vec<_>>(),
        "{input:?}"
    );
}
#[test]
fn scroll_left() {
    let mut t = CoreTerminal::new(5, 5, 1).unwrap();
    t.write(&"12345".repeat(6)).unwrap();
    check(
        &mut t,
        "\x1b[ @",
        &["12345", "2345", "2345", "2345", "2345", "2345"],
    );
    check(
        &mut t,
        "\x1b[0 @",
        &["12345", "345", "345", "345", "345", "345"],
    );
    check(&mut t, "\x1b[2 @", &["12345", "5", "5", "5", "5", "5"]);
}
#[test]
fn scroll_right() {
    let mut t = CoreTerminal::new(5, 5, 1).unwrap();
    t.write(&"12345".repeat(6)).unwrap();
    check(
        &mut t,
        "\x1b[ A",
        &["12345", " 1234", " 1234", " 1234", " 1234", " 1234"],
    );
    check(
        &mut t,
        "\x1b[0 A",
        &["12345", "  123", "  123", "  123", "  123", "  123"],
    );
    check(
        &mut t,
        "\x1b[2 A",
        &["12345", "    1", "    1", "    1", "    1", "    1"],
    );
}
#[test]
fn insert_columns() {
    for (param, result) in [("", "12 34"), ("1", "12 34"), ("2", "12  3")] {
        let mut t = CoreTerminal::new(5, 5, 1).unwrap();
        t.write(&"12345".repeat(6)).unwrap();
        t.write("\x1b[3;3H").unwrap();
        check(
            &mut t,
            &format!("\x1b[{param}'}}"),
            &["12345", result, result, result, result, result],
        );
    }
}
#[test]
fn delete_columns() {
    for (param, result) in [("", "1245"), ("1", "1245"), ("2", "125")] {
        let mut t = CoreTerminal::new(5, 5, 1).unwrap();
        t.write(&"12345".repeat(6)).unwrap();
        t.write("\x1b[3;3H").unwrap();
        check(
            &mut t,
            &format!("\x1b[{param}'~"),
            &["12345", result, result, result, result, result],
        );
    }
}
const TEN_ROWS: &str = "0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7\r\n8\r\n9";
#[test]
fn scroll_up() {
    let mut t = CoreTerminal::new(10, 10, 1000).unwrap();
    t.write(TEN_ROWS).unwrap();
    check(
        &mut t,
        "\x1b[2;4r\x1b[2Sm",
        &["m", "3", "", "", "4", "5", "6", "7", "8", "9"],
    );
}
#[test]
fn scroll_down() {
    let mut t = CoreTerminal::new(10, 10, 1000).unwrap();
    t.write(TEN_ROWS).unwrap();
    check(
        &mut t,
        "\x1b[2;4r\x1b[2Tm",
        &["m", "", "", "1", "4", "5", "6", "7", "8", "9"],
    );
}
#[test]
fn insert_and_delete_lines_outside_margins() {
    for command in ['L', 'M'] {
        let mut t = CoreTerminal::new(10, 10, 1000).unwrap();
        t.write(TEN_ROWS).unwrap();
        t.write("\x1b[3;6r").unwrap();
        let active = t.state().active_buffer();
        let b = active.borrow();
        assert_eq!((b.scroll_top, b.scroll_bottom), (2, 5));
        drop(b);
        check(
            &mut t,
            &format!("\x1b[2{command}m"),
            &["m", "1", "2", "3", "4", "5", "6", "7", "8", "9"],
        );
        check(
            &mut t,
            &format!("\x1b[2H\x1b[2{command}n"),
            &["m", "n", "2", "3", "4", "5", "6", "7", "8", "9"],
        );
        check(
            &mut t,
            &format!("\x1b[7H\x1b[2{command}o"),
            &["m", "n", "2", "3", "4", "5", "o", "7", "8", "9"],
        );
        check(
            &mut t,
            &format!("\x1b[8H\x1b[2{command}p"),
            &["m", "n", "2", "3", "4", "5", "o", "p", "8", "9"],
        );
        check(
            &mut t,
            &format!("\x1b[100H\x1b[2{command}q"),
            &["m", "n", "2", "3", "4", "5", "o", "p", "8", "q"],
        );
    }
}
#[test]
fn insert_lines_within_margins() {
    let mut t = CoreTerminal::new(10, 10, 1000).unwrap();
    t.write(TEN_ROWS).unwrap();
    t.write("\x1b[3;6r").unwrap();
    let active = t.state().active_buffer();
    let b = active.borrow();
    assert_eq!((b.scroll_top, b.scroll_bottom), (2, 5));
    drop(b);
    check(
        &mut t,
        "\x1b[3H\x1b[2Lm",
        &["0", "1", "m", "", "2", "3", "6", "7", "8", "9"],
    );
    check(
        &mut t,
        "\x1b[6H\x1b[2Ln",
        &["0", "1", "m", "", "2", "n", "6", "7", "8", "9"],
    );
}
#[test]
fn delete_lines_within_margins() {
    let mut t = CoreTerminal::new(10, 10, 1000).unwrap();
    t.write(TEN_ROWS).unwrap();
    t.write("\x1b[3;6r").unwrap();
    let active = t.state().active_buffer();
    let b = active.borrow();
    assert_eq!((b.scroll_top, b.scroll_bottom), (2, 5));
    drop(b);
    check(
        &mut t,
        "\x1b[6H\x1b[2Mm",
        &["0", "1", "2", "3", "4", "m", "6", "7", "8", "9"],
    );
    check(
        &mut t,
        "\x1b[3H\x1b[2Mn",
        &["0", "1", "n", "m", "", "", "6", "7", "8", "9"],
    );
}
#[test]
fn repeat_preceding_grapheme() {
    let mut t = CoreTerminal::new(80, 30, 1000).unwrap();
    check(&mut t, "e\u{301}\x1b[2b", &["e\u{301}e\u{301}e\u{301}"]);
}
#[test]
fn repeat_without_predecessor() {
    let mut t = CoreTerminal::new(80, 30, 1000).unwrap();
    check(&mut t, "\x1b[2b", &[""]);
}
#[test]
fn intervening_sequence_resets_repeat() {
    let mut t = CoreTerminal::new(80, 30, 1000).unwrap();
    check(&mut t, "e\u{301}\x1b[0m\x1b[2b", &["e\u{301}"]);
}
#[test]
fn insert_characters_repairs_wide_boundaries() {
    let mut t = CoreTerminal::new(10, 5, 1).unwrap();
    t.write(&"￥".repeat(20)).unwrap();
    check(
        &mut t,
        "\x1b[1;6H\x1b[@",
        &["￥￥   ￥", "￥￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""],
    );
    check(
        &mut t,
        "\x1b[2;4H\x1b[2@",
        &["￥￥   ￥", "￥    ￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""],
    );
    check(
        &mut t,
        "\x1b[3;4H\x1b[3@",
        &["￥￥   ￥", "￥    ￥￥", "￥     ￥", "￥￥￥￥￥", ""],
    );
    check(
        &mut t,
        "\x1b[4;4H\x1b[4@",
        &["￥￥   ￥", "￥    ￥￥", "￥     ￥", "￥      ￥", ""],
    );
}
#[test]
fn delete_characters_repairs_wide_boundaries() {
    let mut t = CoreTerminal::new(10, 5, 1).unwrap();
    t.write(&"￥".repeat(20)).unwrap();
    check(
        &mut t,
        "\x1b[1;6H\x1b[P",
        &["￥￥ ￥￥", "￥￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""],
    );
    check(
        &mut t,
        "\x1b[2;6H\x1b[2P",
        &["￥￥ ￥￥", "￥￥  ￥", "￥￥￥￥￥", "￥￥￥￥￥", ""],
    );
    check(
        &mut t,
        "\x1b[3;6H\x1b[3P",
        &["￥￥ ￥￥", "￥￥  ￥", "￥￥ ￥", "￥￥￥￥￥", ""],
    );
}
#[test]
fn insert_and_delete_characters_regression() {
    for command in ['@', 'P'] {
        let mut t = CoreTerminal::new(80, 30, 1000).unwrap();
        t.write(&"a".repeat(70)).unwrap();
        t.write("1234567890").unwrap();
        t.write(&"a".repeat(70)).unwrap();
        t.write("1234567890").unwrap();
        assert_eq!(
            lines(&t, 1, false),
            vec![format!("{}1234567890", "a".repeat(70))
                .encode_utf16()
                .collect::<Vec<_>>()]
        );
        for (i, param) in [0, 1, 2, 10].iter().enumerate() {
            let active = t.state().active_buffer();
            {
                let mut b = active.borrow_mut();
                b.x = 70;
                b.y = 0;
            }
            t.write(&format!("\x1b[{param}{command}")).unwrap();
            let suffix = if command == '@' {
                [" 123456789", "  12345678", "    123456", "          "][i]
            } else {
                ["234567890 ", "34567890  ", "567890    ", "          "][i]
            };
            assert_eq!(
                lines(&t, 1, false),
                vec![format!("{}{suffix}", "a".repeat(70))
                    .encode_utf16()
                    .collect::<Vec<_>>()]
            );
            if command == 'P' || i == 3 {
                assert_eq!(
                    lines(&t, 1, true),
                    vec![format!("{}{}", "a".repeat(70), suffix.trim_end())
                        .encode_utf16()
                        .collect::<Vec<_>>()]
                );
            }
        }
    }
}
