// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
// Faithful erase assertions from common/InputHandler.test.ts via real CSI parsing.
use xterm_core::{options::OptionValue, terminal::CoreTerminal};
fn cursor(t: &CoreTerminal, x: usize, y: usize) {
    let active = t.state().active_buffer();
    let mut b = active.borrow_mut();
    b.x = x;
    b.y = y;
}
fn row(t: &CoreTerminal, index: usize, trim: bool) -> String {
    let active = t.state().active_buffer();
    let line = active.borrow().lines.get(index).unwrap();
    let value = line.borrow().translate_to_string(trim, None, None, None);
    String::from_utf16(&value).unwrap()
}
fn wrapped(t: &CoreTerminal, index: usize) -> bool {
    let active = t.state().active_buffer();
    let line = active.borrow().lines.get(index).unwrap();
    let value = line.borrow().is_wrapped;
    value
}
fn fill(t: &mut CoreTerminal, rows: usize) {
    let cols = t.state().cols();
    for _ in 0..rows {
        t.write(&"a".repeat(cols)).unwrap();
    }
}
fn wrapped_base(t: &mut CoreTerminal, rows: usize) {
    cursor(t, 0, 0);
    let cols = t.state().cols();
    t.write(&"a".repeat(cols)).unwrap();
    t.write(&"a".repeat(cols + 9)).unwrap();
    for _ in 3..rows {
        t.write(&"a".repeat(cols)).unwrap();
    }
    assert!(wrapped(t, 2));
}

#[test]
fn original_erase_in_line_assertions() {
    let mut t = CoreTerminal::new(80, 30, 1000).unwrap();
    fill(&mut t, 3);
    cursor(&t, 70, 0);
    t.write("\x1b[0K").unwrap();
    assert_eq!(row(&t, 0, false), "a".repeat(70) + &" ".repeat(10));
    cursor(&t, 70, 1);
    t.write("\x1b[1K").unwrap();
    assert_eq!(row(&t, 1, false), " ".repeat(71) + &"a".repeat(9));
    cursor(&t, 70, 2);
    t.write("\x1b[2K").unwrap();
    assert_eq!(row(&t, 2, false), " ".repeat(80));
}

#[test]
fn original_erase_in_line_reflow_assertions() {
    let mut t = CoreTerminal::new(80, 30, 1000).unwrap();
    wrapped_base(&mut t, 30);
    cursor(&t, 40, 2);
    t.write("\x1b[K").unwrap();
    assert!(wrapped(&t, 2));
    cursor(&t, 0, 2);
    t.write("\x1b[K").unwrap();
    assert!(!wrapped(&t, 2));
    wrapped_base(&mut t, 30);
    cursor(&t, 40, 2);
    t.write("\x1b[1K").unwrap();
    assert!(wrapped(&t, 2));
    wrapped_base(&mut t, 30);
    cursor(&t, 40, 2);
    t.write("\x1b[2K").unwrap();
    assert!(!wrapped(&t, 2));
}

#[test]
fn original_ed2_scroll_on_erase_assertions() {
    let mut t = CoreTerminal::new(80, 30, 1000).unwrap();
    t.state()
        .options
        .set("scrollOnEraseInDisplay", OptionValue::Bool(true))
        .unwrap();
    fill(&mut t, 2);
    t.write("\x1b[2J").unwrap();
    let active = t.state().active_buffer();
    assert_eq!(active.borrow().lines.len(), 32);
    assert_eq!(active.borrow().ybase, 2);
    assert_eq!(row(&t, 0, true), "a".repeat(80));
    assert_eq!(row(&t, 1, true), "a".repeat(80));
    cursor(&t, 0, 29);
    fill(&mut t, 1);
    t.write("\x1b[2J").unwrap();
    assert_eq!(active.borrow().lines.len(), 62);
}

#[test]
fn original_erase_in_display_assertions_and_wrap_cases() {
    let mut t = CoreTerminal::new(80, 7, 1000).unwrap();
    fill(&mut t, 7);
    cursor(&t, 40, 5);
    t.write("\x1b[0J").unwrap();
    let expected = [
        "a".repeat(80),
        "a".repeat(80),
        "a".repeat(80),
        "a".repeat(80),
        "a".repeat(80),
        "a".repeat(40) + &" ".repeat(40),
        " ".repeat(80),
    ];
    for (i, line) in expected.iter().enumerate() {
        assert_eq!(row(&t, i, false), *line);
    }
    let expected = [
        "a".repeat(80),
        "a".repeat(80),
        "a".repeat(80),
        "a".repeat(80),
        "a".repeat(80),
        "a".repeat(40),
        String::new(),
    ];
    for (i, line) in expected.iter().enumerate() {
        assert_eq!(row(&t, i, true), *line);
    }
    cursor(&t, 0, 0);
    fill(&mut t, 7);
    cursor(&t, 40, 5);
    t.write("\x1b[1J").unwrap();
    for i in 0..5 {
        assert_eq!(row(&t, i, false), " ".repeat(80));
        assert_eq!(row(&t, i, true), "");
    }
    for trim in [false, true] {
        assert_eq!(row(&t, 5, trim), " ".repeat(41) + &"a".repeat(39));
        assert_eq!(row(&t, 6, trim), "a".repeat(80));
    }
    cursor(&t, 0, 0);
    fill(&mut t, 7);
    cursor(&t, 40, 5);
    t.write("\x1b[2J").unwrap();
    for i in 0..7 {
        assert_eq!(row(&t, i, false), " ".repeat(80));
        assert_eq!(row(&t, i, true), "");
    }
    wrapped_base(&mut t, 7);
    cursor(&t, 40, 2);
    t.write("\x1b[1J").unwrap();
    assert!(!wrapped(&t, 2));
    wrapped_base(&mut t, 7);
    cursor(&t, 90, 1);
    t.write("\x1b[1J").unwrap();
    assert!(!wrapped(&t, 2));
}
