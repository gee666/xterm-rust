// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Remaining common InputHandler erase cases, without changing source assertions.
use xterm_core::terminal::{CoreTerminal, TerminalEvent};
#[path = "support/erase_text.rs"]
mod text;
use text::lines;
fn wide() -> CoreTerminal {
    let mut t = CoreTerminal::new(10, 5, 1).unwrap();
    t.write(&"￥".repeat(20)).unwrap();
    t
}

#[test]
fn original_wide_el_cell_repairs() {
    let mut t = wide();
    t.write("\x1b[1;6H\x1b[K#").unwrap();
    assert_eq!(
        lines(&t, 5),
        ["￥￥ #", "￥￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    t.write("\x1b[2;5H\x1b[1K").unwrap();
    assert_eq!(
        lines(&t, 5),
        ["￥￥ #", "      ￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    t.write("\x1b[3;6H\x1b[1K").unwrap();
    assert_eq!(
        lines(&t, 5),
        ["￥￥ #", "      ￥￥", "      ￥￥", "￥￥￥￥￥", ""]
    );
}
#[test]
fn original_wide_ech_cell_repairs() {
    let mut t = wide();
    t.write("\x1b[1;6H\x1b[X").unwrap();
    assert_eq!(
        lines(&t, 5),
        ["￥￥  ￥￥", "￥￥￥￥￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    t.write("\x1b[2;6H\x1b[2X").unwrap();
    assert_eq!(
        lines(&t, 5),
        ["￥￥  ￥￥", "￥￥    ￥", "￥￥￥￥￥", "￥￥￥￥￥", ""]
    );
    t.write("\x1b[3;6H\x1b[3X").unwrap();
    assert_eq!(
        lines(&t, 5),
        ["￥￥  ￥￥", "￥￥    ￥", "￥￥    ￥", "￥￥￥￥￥", ""]
    );
}
#[test]
fn original_ech_last_cell_and_large_positive_cursor_restriction() {
    let mut t = CoreTerminal::new(10, 10, 1000).unwrap();
    t.write("0123456789\x1b[X").unwrap();
    let active = t.state().active_buffer();
    let line = active.borrow().lines.get(0).unwrap();
    assert_eq!(
        String::from_utf16(&line.borrow().translate_to_string(false, None, None, None)).unwrap(),
        "012345678 "
    );
    {
        let mut b = active.borrow_mut();
        b.x = 10000;
        b.y = 10000;
    }
    t.write("\x1b[X").unwrap();
    let b = active.borrow();
    assert_eq!((b.x, b.y), (9, 9));
    // The original negative private-coordinate half is tracked explicitly by
    // signed cursor task152. Buffer's usize fields cannot represent that state.
}
#[test]
fn original_pending_wrap_ed_el_cursor_positions_and_rows() {
    for (sequence, retained) in [
        ("\x1b[0K", true),
        ("\x1b[1K", false),
        ("\x1b[2K", false),
        ("\x1b[0J", true),
        ("\x1b[1J", false),
        ("\x1b[2J", false),
        ("\x1b[3J", true),
    ] {
        let mut t = CoreTerminal::new(10, 5, 1000).unwrap();
        t.write(&format!("##########{sequence}")).unwrap();
        let active = t.state().active_buffer();
        assert_eq!(active.borrow().x, 10);
        assert_eq!(
            lines(&t, 5),
            [if retained { "##########" } else { "" }, "", "", "", ""]
        );
    }
}
#[test]
fn original_following_sequences_never_advance_past_columns() {
    let sequences = [
        "\x1b[10@",
        "\x1b[10 @",
        "\x1b[10A",
        "\x1b[10 A",
        "\x1b[10B",
        "\x1b[10C",
        "\x1b[10D",
        "\x1b[10E",
        "\x1b[10F",
        "\x1b[10G",
        "\x1b[10;10H",
        "\x1b[10I",
        "\x1b[10L",
        "\x1b[10M",
        "\x1b[10P",
        "\x1b[10S",
        "\x1b[10T",
        "\x1b[10X",
        "\x1b[10Z",
        "\x1b[10`",
        "\x1b[10a",
        "\x1b[10b",
        "\x1b[10d",
        "\x1b[10e",
        "\x1b[10;10f",
        "\x1b[0g",
        "\x1b[s",
        "\x1b[10'}",
        "\x1b[10'~",
    ];
    for sequence in sequences {
        let mut t = CoreTerminal::new(10, 5, 1000).unwrap();
        t.write(&format!("##########\x1b[2J{sequence}")).unwrap();
        assert!(t.state().active_buffer().borrow().x <= 10, "{sequence:?}");
    }
}
#[test]
fn original_ed3_unlocks_normal_viewport_and_future_scroll_follows_bottom() {
    let mut t = CoreTerminal::new(10, 5, 1000).unwrap();
    for i in 0..20 {
        t.write(&format!("old {i}\r\n")).unwrap();
    }
    t.scroll_lines(-5).unwrap();
    assert!(t.state().buffer_service.is_user_scrolling.get());
    t.write("\x1b[3J").unwrap();
    let active = t.state().active_buffer();
    assert!(!t.state().buffer_service.is_user_scrolling.get());
    assert_eq!((active.borrow().ybase, active.borrow().ydisp), (0, 0));
    for i in 0..20 {
        t.write(&format!("new {i}\r\n")).unwrap();
    }
    assert_eq!(active.borrow().ydisp, active.borrow().ybase);
}
#[test]
fn original_alt_ed3_preserves_normal_user_viewport_after_resizing() {
    let mut t = CoreTerminal::new(10, 5, 1000).unwrap();
    t.resize(10, 50).unwrap();
    t.resize(10, 5).unwrap();
    for i in 0..20 {
        t.write(&format!("old {i}\r\n")).unwrap();
    }
    t.scroll_lines(-5).unwrap();
    let normal = t.state().active_buffer();
    let ydisp = normal.borrow().ydisp;
    t.write("\x1b[?1049h").unwrap();
    for i in 0..20 {
        t.write(&format!("new {i}\r\n")).unwrap();
    }
    t.write("\x1b[3J\x1b[?1049l").unwrap();
    assert!(t.state().buffer_service.is_user_scrolling.get());
    assert_eq!(normal.borrow().ydisp, ydisp);
    t.write("more\r\n").unwrap();
    assert!(t.state().buffer_service.is_user_scrolling.get());
    assert_eq!(normal.borrow().ydisp, ydisp);
}
#[test]
fn original_decsca_default_and_selective_line_display_erasure() {
    let mut t = CoreTerminal::new(80, 25, 1000).unwrap();
    t.write("some text\x1b[?2K").unwrap();
    assert_eq!(lines(&t, 2), ["", ""]);
    t.write("some text\x1b[?2J").unwrap();
    assert_eq!(lines(&t, 2), ["", ""]);
    for (protected, command) in [("lineerase", "K"), ("displayerase", "J")] {
        let mut t = CoreTerminal::new(80, 25, 1000).unwrap();
        t.write(&format!(
            "###\x1b[1\"q{protected}\x1b[0\"q***\x1b[?2{command}"
        ))
        .unwrap();
        assert_eq!(lines(&t, 2), [format!("   {protected}"), String::new()]);
        t.write(&format!("\x1b[2{command}")).unwrap();
        assert_eq!(lines(&t, 2), ["", ""]);
    }
}
#[test]
fn original_decrqss_reports_protection_default_set_and_reset() {
    let mut t = CoreTerminal::new(80, 25, 1000).unwrap();
    for (prefix, expected) in [
        ("", "\x1bP1$r0\"q\x1b\\"),
        ("###\x1b[1\"q", "\x1bP1$r1\"q\x1b\\"),
        ("###\x1b[2\"q", "\x1bP1$r0\"q\x1b\\"),
    ] {
        t.write(&format!("{prefix}\x1bP$q\"q\x1b\\")).unwrap();
        let replies: Vec<_> = t
            .drain_events()
            .into_iter()
            .filter_map(|e| {
                if let TerminalEvent::Data(s) = e {
                    Some(String::from_utf16(&s).unwrap())
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(replies, [expected]);
    }
}
