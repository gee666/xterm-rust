// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
// Faithful InputHandler.test.ts setMode, DECRQM and Kitty negotiation assertions.
use std::rc::Rc;
use xterm_core::{
    options::OptionValue as Value,
    terminal::{CoreTerminal, TerminalEvent},
};

fn terminal() -> CoreTerminal {
    CoreTerminal::new(80, 30, 1000).unwrap()
}
fn reply(terminal: &mut CoreTerminal, input: &str) -> String {
    terminal.write(input).unwrap();
    terminal
        .drain_events()
        .into_iter()
        .filter_map(|event| {
            if let TerminalEvent::Data(units) = event {
                Some(String::from_utf16(&units).unwrap())
            } else {
                None
            }
        })
        .last()
        .expect("source query emits a data event")
}
fn options(terminal: &CoreTerminal, key: &str, entries: &[(&str, bool)]) {
    terminal
        .state()
        .options
        .set(
            key,
            Value::property_map(
                entries
                    .iter()
                    .map(|(key, value)| (*key, Value::Bool(*value))),
            ),
        )
        .unwrap();
}

#[test]
fn source_toggle_bracketed_paste() {
    let mut t = terminal();
    t.write("\x1b[?2004h").unwrap();
    assert!(
        t.state()
            .core
            .dec_private_modes
            .borrow()
            .bracketed_paste_mode
    );
    t.write("\x1b[?2004l").unwrap();
    assert!(
        !t.state()
            .core
            .dec_private_modes
            .borrow()
            .bracketed_paste_mode
    );
}
#[test]
fn source_toggle_color_scheme_and_disabled_gate() {
    let mut t = terminal();
    t.write("\x1b[?2031h").unwrap();
    assert!(
        t.state()
            .core
            .dec_private_modes
            .borrow()
            .color_scheme_updates
    );
    t.write("\x1b[?2031l").unwrap();
    assert!(
        !t.state()
            .core
            .dec_private_modes
            .borrow()
            .color_scheme_updates
    );
    options(&t, "vtExtensions", &[("colorSchemeQuery", false)]);
    t.write("\x1b[?2031h").unwrap();
    assert!(
        !t.state()
            .core
            .dec_private_modes
            .borrow()
            .color_scheme_updates
    );
}
#[test]
fn source_ansi_keyboard_action_permanently_reset() {
    assert_eq!(reply(&mut terminal(), "\x1b[2$p"), "\x1b[2;4$y");
}
#[test]
fn source_ansi_insert_mode() {
    let mut t = terminal();
    assert_eq!(reply(&mut t, "\x1b[4$p"), "\x1b[4;2$y");
    assert_eq!(reply(&mut t, "\x1b[4h\x1b[4$p"), "\x1b[4;1$y");
    assert_eq!(reply(&mut t, "\x1b[4l\x1b[4$p"), "\x1b[4;2$y");
}
#[test]
fn source_ansi_send_receive_permanently_set() {
    assert_eq!(reply(&mut terminal(), "\x1b[12$p"), "\x1b[12;3$y");
}
#[test]
fn source_ansi_newline_mode() {
    let mut t = terminal();
    assert_eq!(reply(&mut t, "\x1b[20$p"), "\x1b[20;2$y");
    assert_eq!(reply(&mut t, "\x1b[20h\x1b[20$p"), "\x1b[20;1$y");
    assert_eq!(reply(&mut t, "\x1b[20l\x1b[20$p"), "\x1b[20;2$y");
}
#[test]
fn source_ansi_unknown_mode() {
    assert_eq!(reply(&mut terminal(), "\x1b[1234$p"), "\x1b[1234;0$y");
}
#[test]
fn source_dec_private_set_reset_semantics() {
    let mut t = terminal();
    for mode in [
        1, 6, 9, 45, 66, 1000, 1002, 1003, 1004, 1006, 1016, 47, 1047, 1049, 2004, 2026,
    ] {
        let query = format!("\x1b[?{mode}$p");
        assert_eq!(reply(&mut t, &query), format!("\x1b[?{mode};2$y"));
        assert_eq!(
            reply(&mut t, &format!("\x1b[?{mode}h{query}")),
            format!("\x1b[?{mode};1$y")
        );
        assert_eq!(
            reply(&mut t, &format!("\x1b[?{mode}l{query}")),
            format!("\x1b[?{mode};2$y")
        );
    }
    for mode in [7, 25] {
        let query = format!("\x1b[?{mode}$p");
        assert_eq!(reply(&mut t, &query), format!("\x1b[?{mode};1$y"));
        assert_eq!(
            reply(&mut t, &format!("\x1b[?{mode}l{query}")),
            format!("\x1b[?{mode};2$y")
        );
        assert_eq!(
            reply(&mut t, &format!("\x1b[?{mode}h{query}")),
            format!("\x1b[?{mode};1$y")
        );
    }
}
#[test]
fn source_dec_cursor_blink_quirk_gate() {
    let mut t = terminal();
    assert_eq!(reply(&mut t, "\x1b[?12$p"), "\x1b[?12;2$y");
    assert_eq!(reply(&mut t, "\x1b[?12h\x1b[?12$p"), "\x1b[?12;2$y");
    options(&t, "quirks", &[("allowSetCursorBlink", true)]);
    assert_eq!(reply(&mut t, "\x1b[?12h\x1b[?12$p"), "\x1b[?12;1$y");
    assert_eq!(reply(&mut t, "\x1b[?12l\x1b[?12$p"), "\x1b[?12;2$y");
}
#[test]
fn source_dec_permanent_modes() {
    let mut t = terminal();
    for (mode, value) in [(3, 0), (8, 3), (67, 4), (1005, 4), (1015, 4), (1048, 1)] {
        assert_eq!(
            reply(&mut t, &format!("\x1b[?{mode}$p")),
            format!("\x1b[?{mode};{value}$y")
        );
    }
}
#[test]
fn source_kitty_stack_limit_evicts_oldest() {
    let mut t = terminal();
    options(&t, "vtExtensions", &[("kittyKeyboard", true)]);
    for flags in 1..=20 {
        t.write(&format!("\x1b[>{flags}u")).unwrap();
    }
    assert_eq!(t.state().core.kitty_keyboard.borrow().main_stack.len(), 16);
    assert_eq!(t.state().core.kitty_keyboard.borrow().main_stack[0], 4);
}
#[test]
fn source_kitty_buffer_switch_has_separate_flags() {
    let mut t = terminal();
    options(&t, "vtExtensions", &[("kittyKeyboard", true)]);
    t.write("\x1b[>5u").unwrap();
    assert_eq!(t.state().core.kitty_keyboard.borrow().flags, 5);
    t.write("\x1b[?1049h").unwrap();
    assert_eq!(t.state().core.kitty_keyboard.borrow().flags, 0);
    assert_eq!(t.state().core.kitty_keyboard.borrow().main_flags, 5);
    t.write("\x1b[>7u").unwrap();
    assert_eq!(t.state().core.kitty_keyboard.borrow().flags, 7);
    t.write("\x1b[?1049l").unwrap();
    assert_eq!(t.state().core.kitty_keyboard.borrow().flags, 5);
    assert_eq!(t.state().core.kitty_keyboard.borrow().alt_flags, 7);
}
#[test]
fn source_kitty_pop_resets_when_stack_emptied() {
    let mut t = terminal();
    options(&t, "vtExtensions", &[("kittyKeyboard", true)]);
    t.write("\x1b[>5u").unwrap();
    assert_eq!(t.state().core.kitty_keyboard.borrow().flags, 5);
    t.write("\x1b[<10u").unwrap();
    assert_eq!(t.state().core.kitty_keyboard.borrow().flags, 0);
}
#[test]
fn kitty_set_or_clear_query_disable_and_chunking() {
    let mut t = terminal();
    t.write("\x1b[=7u\x1b[>9u\x1b[<u\x1b[?u").unwrap();
    assert!(t
        .drain_events()
        .iter()
        .all(|event| !matches!(event, TerminalEvent::Data(_))));
    options(&t, "vtExtensions", &[("kittyKeyboard", true)]);
    assert_eq!(reply(&mut t, "\x1b[=5u\x1b[?u"), "\x1b[?5u");
    assert_eq!(reply(&mut t, "\x1b[=2;2u\x1b[?u"), "\x1b[?7u");
    assert_eq!(reply(&mut t, "\x1b[=1;3u\x1b[?u"), "\x1b[?6u");
    assert_eq!(reply(&mut t, "\x1b[=9;99u\x1b[?u"), "\x1b[?6u");
    for chunk in ["\x1b", "[=", "11", ";0", "u"] {
        t.write(chunk).unwrap();
    }
    assert_eq!(reply(&mut t, "\x1b[?u"), "\x1b[?11u");
}
#[test]
fn query_only_reads_first_parameter_and_mouse_modes_are_mutually_exclusive() {
    let mut t = terminal();
    assert_eq!(reply(&mut t, "\x1b[4;20$p"), "\x1b[4;2$y");
    t.write("\x1b[?9;1000;1002;1003;1006;1016h").unwrap();
    assert_eq!(t.state().mouse.active_protocol(), "ANY");
    assert_eq!(t.state().mouse.active_encoding(), "SGR_PIXELS");
    t.write("\x1b[?9;1006l").unwrap();
    assert_eq!(t.state().mouse.active_protocol(), "NONE");
    assert_eq!(t.state().mouse.active_encoding(), "DEFAULT");
}
#[test]
fn gated_win32_and_column_mode_queries() {
    let mut t = terminal();
    assert_eq!(reply(&mut t, "\x1b[?9001h\x1b[?9001$p"), "\x1b[?9001;0$y");
    options(&t, "vtExtensions", &[("win32InputMode", true)]);
    assert_eq!(reply(&mut t, "\x1b[?9001h\x1b[?9001$p"), "\x1b[?9001;1$y");
    assert_eq!(reply(&mut t, "\x1b[?9001l\x1b[?9001$p"), "\x1b[?9001;2$y");
    options(&t, "windowOptions", &[("setWinLines", true)]);
    assert_eq!(reply(&mut t, "\x1b[?3h\x1b[?3$p"), "\x1b[?3;1$y");
    assert_eq!(t.cols(), 132);
    assert_eq!(reply(&mut t, "\x1b[?3l\x1b[?3$p"), "\x1b[?3;2$y");
    assert_eq!(t.cols(), 80);
    t.resize(91, 30).unwrap();
    assert_eq!(reply(&mut t, "\x1b[?3$p"), "\x1b[?3;0$y");
}
#[test]
fn optional_option_bags_null_preserve_source_defaults() {
    let mut t = terminal();
    let options = Rc::clone(&t.state().options);
    options.set("vtExtensions", Value::Null).unwrap();
    options.set("quirks", Value::Undefined).unwrap();
    t.write("\x1b[?2031h\x1b[?12h\x1b[=7u").unwrap();
    assert!(
        t.state()
            .core
            .dec_private_modes
            .borrow()
            .color_scheme_updates
    );
    assert!(!t.state().options.get("cursorBlink").unwrap().is_truthy());
    assert_eq!(t.state().core.kitty_keyboard.borrow().flags, 0);
}
