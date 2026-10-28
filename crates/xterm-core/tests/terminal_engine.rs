// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use xterm_core::terminal::{CoreTerminal, TerminalEvent};
fn lines(terminal: &CoreTerminal) -> Vec<String> {
    let state = terminal.state();
    let active = state.active_buffer();
    let b = active.borrow();
    (0..b.lines.len())
        .map(|i| {
            String::from_utf16_lossy(
                &b.lines
                    .get(i)
                    .unwrap()
                    .borrow()
                    .translate_to_string(true, None, None, None),
            )
        })
        .collect()
}
#[test]
fn print_wrap_scroll_and_explicit_linefeed() {
    let mut t = CoreTerminal::new(4, 2, 2).unwrap();
    t.write("abcdefghi").unwrap();
    assert_eq!(lines(&t), ["abcd", "efgh", "i"]);
    {
        let s = t.state();
        let b = s.active_buffer();
        let b = b.borrow();
        assert_eq!((b.x, b.y, b.ybase, b.ydisp), (1, 1, 1, 1));
    }
    t.write("\r\nz").unwrap();
    assert_eq!(lines(&t), ["abcd", "efgh", "i", "z"]);
    let events = t.drain_events();
    assert!(events.contains(&TerminalEvent::LineFeed));
    assert!(events.contains(&TerminalEvent::Scroll(2)));
}
#[test]
fn decoder_boundaries_wide_combining_and_bom() {
    let mut t = CoreTerminal::new(6, 2, 0).unwrap();
    for byte in "a界e\u{301}🙂".as_bytes() {
        t.write_bytes(&[*byte]).unwrap();
    }
    assert_eq!(lines(&t), ["a界e\u{301}🙂", ""]);
    {
        let s = t.state();
        let b = s.active_buffer();
        let b = b.borrow();
        let l = b.lines.get(0).unwrap();
        let l = l.borrow();
        assert_eq!(l.get_width(1), Some(2));
        assert_eq!(l.get_width(2), Some(0));
        assert_eq!(l.get_string(3), Some(vec![b'e' as u16, 0x301]));
    }
    let mut t = CoreTerminal::new(4, 2, 0).unwrap();
    t.write_utf16(&[0xd83d]).unwrap();
    t.write_utf16(&[0xde42, 0xfeff]).unwrap();
    assert_eq!(lines(&t), ["🙂", ""]);
}
#[test]
fn charset_designation_shift_and_saved_cursor() {
    let mut t = CoreTerminal::new(8, 2, 0).unwrap();
    t.write("\x1b)0\x0eq\x0fx\x1b7abc\x1b8Z").unwrap();
    assert_eq!(lines(&t), ["─xZbc", ""]);
}
#[test]
fn control_input_does_not_echo_and_preserves_events() {
    let mut t = CoreTerminal::new(4, 2, 0).unwrap();
    t.write("ab\x08Z\r!\x07\x1b]2;hello\x07").unwrap();
    assert_eq!(lines(&t), ["!Z", ""]);
    t.input(&[0xd800, 13], true).unwrap();
    let events = t.drain_events();
    assert!(events.contains(&TerminalEvent::Bell));
    assert!(events.contains(&TerminalEvent::TitleChange(
        "hello".encode_utf16().collect()
    )));
    assert!(events.contains(&TerminalEvent::Data(vec![0xd800, 13])));
    assert_eq!(lines(&t), ["!Z", ""]);
}
#[test]
fn direct_reset_retains_parser_partial_sequence() {
    let mut t = CoreTerminal::new(4, 2, 0).unwrap();
    t.write("\x1b]2;before").unwrap();
    t.reset().unwrap();
    t.write("after\x07").unwrap();
    assert_eq!(
        t.state().title,
        "beforeafter".encode_utf16().collect::<Vec<_>>()
    );
    assert_eq!(lines(&t), ["", ""]);
}
#[test]
fn event_callback_observes_action_time_and_can_reset_before_next_print() {
    use std::{cell::RefCell, rc::Rc};
    let mut t = CoreTerminal::new(4, 2, 0).unwrap();
    let state = t.state_handle();
    let observations = Rc::new(RefCell::new(Vec::new()));
    let saved = Rc::clone(&observations);
    t.set_event_handler(Some(Rc::new(move |event| {
        if event == TerminalEvent::Bell {
            let active = state.borrow().active_buffer();
            let b = active.borrow();
            saved.borrow_mut().push(String::from_utf16_lossy(
                &b.lines
                    .get(0)
                    .unwrap()
                    .borrow()
                    .translate_to_string(true, None, None, None),
            ));
            drop(b);
            xterm_core::terminal::reset_state_handle(&state).unwrap();
        }
        Ok(())
    })));
    t.write("A\x07B").unwrap();
    assert_eq!(*observations.borrow(), ["A"]);
    assert_eq!(lines(&t), ["B", ""]);
}
#[test]
fn replies_follow_parse_order_and_do_not_print() {
    let mut t = CoreTerminal::new(4, 2, 0).unwrap();
    t.write("a\x1b[c\x1b[5n\x1b[6n\x1bP$qm\x1b\\").unwrap();
    let data: Vec<_> = t
        .drain_events()
        .into_iter()
        .filter_map(|e| {
            if let TerminalEvent::Data(d) = e {
                Some(String::from_utf16_lossy(&d))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(
        data,
        ["\x1b[?1;2c", "\x1b[0n", "\x1b[1;2R", "\x1bP1$r0m\x1b\\"]
    );
    assert_eq!(lines(&t), ["a", ""]);
}
#[test]
fn clear_at_first_row_and_dispose_are_real_operations() {
    let mut t = CoreTerminal::new(4, 2, 0).unwrap();
    t.write("abc").unwrap();
    t.clear().unwrap();
    assert_eq!(lines(&t), ["abc", ""]);
    assert!(t.drain_events().contains(&TerminalEvent::Scroll(0)));
    t.dispose().unwrap();
    assert!(t.write("lost").is_err());
    t.dispose().unwrap();
}
