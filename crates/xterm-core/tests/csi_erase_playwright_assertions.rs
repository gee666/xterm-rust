// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Exact content assertions from test/playwright/InputHandler.test.ts. These run
// the real Rust engine, but do not substitute for actual WASM/browser execution.
#[path = "support/erase_text.rs"]
mod text;
use text::lines;
use xterm_core::terminal::CoreTerminal;
#[test]
fn original_browser_ed_and_decsed_content_assertions() {
    for prefix in ["", "?"] {
        let fixture = "abc\n\rdef\n\rghi\x1b[2;2H";
        for (mode, expected) in [
            ("", ["abc", "d", ""]),
            ("0", ["abc", "d", ""]),
            ("1", ["", "  f", "ghi"]),
        ] {
            let mut t = CoreTerminal::new(5, 5, 1000).unwrap();
            t.write(&format!("{fixture}\x1b[{prefix}{mode}J")).unwrap();
            assert_eq!(lines(&t, 3), expected, "ED prefix {prefix:?} mode {mode:?}");
        }
        let mut t = CoreTerminal::new(5, 5, 1000).unwrap();
        t.write(&format!("1\n2\n3\n4\n5{fixture}\x1b[{prefix}3J"))
            .unwrap();
        assert_eq!(t.state().active_buffer().borrow().lines.len(), 5);
        assert_eq!(lines(&t, 5), ["   4", "    5", "abc", "def", "ghi"]);
    }
}
#[test]
fn original_browser_el_and_decsel_content_assertions() {
    for prefix in ["", "?"] {
        for (mode, expected) in [("", "ab"), ("0", "ab"), ("1", "   de"), ("2", "")] {
            let mut t = CoreTerminal::new(80, 24, 1000).unwrap();
            t.write(&format!("abcde\x1b[1;3H\x1b[{prefix}{mode}K"))
                .unwrap();
            assert_eq!(
                lines(&t, 1),
                [expected],
                "EL prefix {prefix:?} mode {mode:?}"
            );
        }
    }
}
#[test]
fn original_browser_ech_content_assertions() {
    for (count, expected) in [("", " bcdef"), ("3", "   def")] {
        let mut t = CoreTerminal::new(80, 24, 1000).unwrap();
        t.write(&format!("abcdef\x1b[1;1H\x1b[{count}X")).unwrap();
        assert_eq!(lines(&t, 1), [expected]);
    }
}
#[test]
fn original_browser_decsca_content_assertions() {
    let mut t = CoreTerminal::new(80, 24, 1000).unwrap();
    t.write("\x1b[1\"qPROT\x1b[2\"qopen").unwrap();
    assert_eq!(lines(&t, 1), ["PROTopen"]);
    t.write("\x1b[1;1H\x1b[?2K").unwrap();
    assert_eq!(lines(&t, 1), ["PROT"]);
}
