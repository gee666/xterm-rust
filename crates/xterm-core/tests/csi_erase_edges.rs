// Copyright (c) 2014-2026 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, error::Error, rc::Rc};
use xterm_core::{event::EmitError, terminal::CoreTerminal};

#[test]
fn marker_callbacks_observe_filled_row_before_wrap_clears_without_buffer_borrow() {
    let mut terminal = CoreTerminal::new(4, 3, 4).unwrap();
    terminal.write("abcdefgh").unwrap();
    let active = terminal.state().active_buffer();
    let marker = active.borrow_mut().add_marker(1);
    let observations = Rc::new(RefCell::new(Vec::new()));
    let target = Rc::clone(&observations);
    let buffer = Rc::downgrade(&active);
    marker.on_dispose().subscribe(move |_| {
        let active = buffer.upgrade().unwrap();
        let b = active
            .try_borrow()
            .expect("marker observer can read buffer");
        let line = b.lines.get(1).unwrap();
        let line = line.borrow();
        target
            .borrow_mut()
            .push((line.is_wrapped, line.get_trimmed_length()));
    });
    terminal.write("\x1b[2J").unwrap();
    assert_eq!(*observations.borrow(), [(true, 0)]);
    assert!(!active.borrow().lines.get(1).unwrap().borrow().is_wrapped);
    assert!(marker.is_disposed());
}

#[test]
fn marker_callback_exception_stops_before_wrap_clear_and_retains_original_cause() {
    let mut terminal = CoreTerminal::new(4, 3, 4).unwrap();
    terminal.write("abcdefgh").unwrap();
    let active = terminal.state().active_buffer();
    let marker = active.borrow_mut().add_marker(1);
    let original: Rc<dyn Error> = Rc::new(std::io::Error::other("erase marker sentinel"));
    let cause = Rc::clone(&original);
    marker
        .on_dispose()
        .try_subscribe(move |_| Err(EmitError::Callback(Rc::clone(&cause))));
    let error = terminal.write("\x1b[2JZ").unwrap_err();
    let mut source: &dyn Error = &error;
    while let Some(next) = source.source() {
        source = next;
    }
    assert!(std::ptr::eq(source, original.as_ref()));
    let line = active.borrow().lines.get(1).unwrap();
    let line = line.borrow();
    assert!(line.is_wrapped);
    assert_eq!(line.get_trimmed_length(), 0);
    assert!(marker.is_disposed());
}

#[test]
fn ed3_trim_observers_see_old_bases_with_new_lines_without_borrow_conflict() {
    let mut terminal = CoreTerminal::new(4, 2, 4).unwrap();
    terminal.write("a\r\nb\r\nc\r\nd").unwrap();
    let active = terminal.state().active_buffer();
    let lines = active.borrow().lines.share();
    let target = Rc::new(RefCell::new(Vec::new()));
    let values = Rc::clone(&target);
    let buffer = Rc::downgrade(&active);
    lines.on_trim_emitter.event().subscribe(move |amount| {
        let active = buffer.upgrade().unwrap();
        let b = active.try_borrow().expect("trim observer can read buffer");
        values
            .borrow_mut()
            .push((*amount, b.ybase, b.ydisp, b.lines.len()));
    });
    terminal.write("\x1b[3J").unwrap();
    assert_eq!(*target.borrow(), [(2, 2, 2, 2)]);
    let b = active.borrow();
    assert_eq!((b.ybase, b.ydisp), (0, 0));
}

#[test]
fn ed3_emits_private_browser_scroll_notification_not_public_scroll() {
    use xterm_core::terminal::TerminalEvent;
    let mut terminal = CoreTerminal::new(4, 2, 4).unwrap();
    terminal.write("a\r\nb\r\nc\r\nd").unwrap();
    terminal.drain_events();
    terminal.write("\x1b[3J").unwrap();
    let events = terminal.drain_events();
    assert!(events.contains(&TerminalEvent::InputScroll(0)));
    assert!(!events
        .iter()
        .any(|event| matches!(event, TerminalEvent::Scroll(_))));
}
