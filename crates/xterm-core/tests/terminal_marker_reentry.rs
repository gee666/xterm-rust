// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::rc::Rc;
use xterm_core::headless::Terminal;

#[test]
fn erase_display_marker_can_reset_public_terminal_mid_action() {
    let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
    terminal.write("abcde12345").unwrap();
    let old = terminal.with_state(|s| s.active_buffer()).unwrap();
    let marker = terminal.register_marker(0).unwrap();
    let weak = Rc::downgrade(&terminal);
    marker.on_dispose().try_subscribe(move |_| {
        weak.upgrade()
            .unwrap()
            .reset()
            .map_err(xterm_core::event::EmitError::callback)
    });
    terminal.write("\x1b[2Jafter").unwrap();
    let active = terminal.with_state(|s| s.active_buffer()).unwrap();
    assert!(!Rc::ptr_eq(&old, &active));
    let row = active.borrow().lines.get(0).unwrap();
    assert_eq!(
        row.borrow().translate_to_string(true, None, None, None),
        "after".encode_utf16().collect::<Vec<_>>()
    );
    assert!(marker.is_disposed());
}

#[test]
fn line_edits_resume_on_replacement_buffer_after_marker_reset() {
    for command in ['L', 'M'] {
        let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
        terminal
            .write("0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7")
            .unwrap();
        let old = terminal.with_state(|s| s.active_buffer()).unwrap();
        assert_eq!(old.borrow().ybase, 5);
        let marker = terminal.register_marker(0).unwrap();
        let weak = Rc::downgrade(&terminal);
        marker.on_dispose().try_subscribe(move |_| {
            weak.upgrade()
                .unwrap()
                .reset()
                .map_err(xterm_core::event::EmitError::callback)
        });
        terminal.write(&format!("\x1b[{command}")).unwrap();
        let active = terminal.with_state(|s| s.active_buffer()).unwrap();
        assert!(!Rc::ptr_eq(&old, &active));
        let b = active.borrow();
        assert_eq!(
            (b.x, b.y, b.ybase, b.ydisp, b.lines.len()),
            (0, 0, 0, 0, 4),
            "{command}"
        );
        assert!(
            b.lines.get(3).is_none(),
            "cached insertion index must preserve source sparse slot: {command}"
        );
    }
}

#[test]
fn region_scrolling_allows_marker_reset_without_state_borrow() {
    for command in ['S', 'T', '^'] {
        let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
        terminal
            .write("0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7")
            .unwrap();
        let marker = terminal
            .register_marker(if command == 'S' { -2 } else { 0 })
            .unwrap();
        let weak = Rc::downgrade(&terminal);
        marker.on_dispose().try_subscribe(move |_| {
            weak.upgrade()
                .unwrap()
                .reset()
                .map_err(xterm_core::event::EmitError::callback)
        });
        terminal.write(&format!("\x1b[{command}")).unwrap();
        let active = terminal.with_state(|s| s.active_buffer()).unwrap();
        let b = active.borrow();
        assert_eq!(
            (b.x, b.y, b.ybase, b.ydisp, b.lines.len()),
            (0, 0, 0, 0, 4),
            "{command}"
        );
        assert!((0..4).all(|i| b.lines.get(i).is_some()), "{command}");
    }
}
