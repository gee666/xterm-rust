// Copyright (c) 2017-2026 The xterm.js authors. All rights reserved. MIT license.
use super::super::TerminalEvent;
use super::{permit, replies, terminal, write, V};
#[test]
fn directed_status_string_unknown_raw_style_reports_nan() {
    let mut t = terminal(80, 30);
    t.state()
        .options
        .set_raw("cursorStyle", V::from("not-a-style"))
        .unwrap();
    write(&mut t, "\x1bP$q q\x1b\\");
    assert_eq!(replies(&mut t), ["\x1bP1$rNaN q\x1b\\"]);
}
#[test]
fn directed_soft_reset_callback_sees_prefix_and_error_stops_suffix() {
    use std::{cell::RefCell, rc::Rc};
    let mut t = terminal(8, 5);
    write(&mut t, "\x1b[4h\x1b[?25l\x1b[2;4r\x1b[1m");
    let state = t.state_handle();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&seen);
    t.set_event_handler(Some(Rc::new(move |event| {
        if event == TerminalEvent::SyncScrollBar {
            let s = state.try_borrow().unwrap();
            let active = s.active_buffer();
            let b = active.borrow();
            output.borrow_mut().push((
                s.core.is_cursor_hidden.get(),
                s.core.modes.borrow().insert_mode,
                s.attributes.is_bold() != 0,
                b.scroll_top,
                b.scroll_bottom,
            ));
            return Err(crate::event::EmitError::ListenerArrayCleared);
        }
        Ok(())
    })));
    assert!(t.write("\x1b[!p").is_err());
    assert_eq!(*seen.borrow(), [(false, true, true, 1, 3)]);
    assert!(t.state().core.modes.borrow().insert_mode);
    assert_ne!(t.state().attributes.is_bold(), 0);
}
#[test]
fn directed_window_title_pop_observes_previous_icon() {
    use std::{cell::RefCell, rc::Rc};
    let mut t = terminal(8, 5);
    permit(&t, &["pushTitle", "popTitle"]);
    write(
        &mut t,
        "\x1b]0;saved\x07\x1b[22t\x1b]0;current\x07\x1b]1;new-icon\x07",
    );
    t.drain_events();
    let state = t.state_handle();
    let seen = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&seen);
    t.set_event_handler(Some(Rc::new(move |event| {
        if let TerminalEvent::TitleChange(title) = event {
            output.borrow_mut().push((
                String::from_utf16_lossy(&title),
                String::from_utf16_lossy(&state.try_borrow().unwrap().icon_name),
            ));
        }
        Ok(())
    })));
    write(&mut t, "\x1b[23t");
    assert_eq!(
        *seen.borrow(),
        [("saved".to_owned(), "new-icon".to_owned())]
    );
    assert_eq!(String::from_utf16_lossy(&t.state().icon_name), "saved");
}
