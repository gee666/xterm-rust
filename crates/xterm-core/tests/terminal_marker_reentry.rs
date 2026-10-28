// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use std::{cell::RefCell, rc::Rc};
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
fn display_erase_modes_allow_public_marker_reset() {
    for (command, offset) in [("0J", 1), ("1J", -1), ("2J", 0), ("?2J", 0), ("3J", -5)] {
        let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
        terminal
            .write("0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7")
            .unwrap();
        if command == "0J" {
            terminal.write("\x1b[H").unwrap();
        }
        let old = terminal.with_state(|s| s.active_buffer()).unwrap();
        let marker = terminal.register_marker(offset).unwrap();
        let weak = Rc::downgrade(&terminal);
        marker.on_dispose().try_subscribe(move |_| {
            weak.upgrade()
                .unwrap()
                .reset()
                .map_err(xterm_core::event::EmitError::callback)
        });
        terminal.write(&format!("\x1b[{command}")).unwrap();
        assert!(marker.is_disposed(), "{command}");
        let active = terminal.with_state(|s| s.active_buffer()).unwrap();
        assert!(!Rc::ptr_eq(&old, &active), "{command}");
        let b = active.borrow();
        assert_eq!(
            (b.x, b.y, b.ybase, b.ydisp, b.lines.len()),
            (0, 0, 0, 0, 3),
            "{command}"
        );
        assert!((0..3).all(|i| b.lines.get(i).unwrap().borrow().get_trimmed_length() == 0));
    }
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

#[test]
fn marker_callback_can_change_options_and_register_another_marker() {
    use xterm_core::options::OptionValue;
    for command in ["2J", "L", "M", "S", "T"] {
        let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
        terminal.write("a\r\nb\r\nc").unwrap();
        let marker = terminal
            .register_marker(if command == "S" { -2 } else { 0 })
            .unwrap();
        let weak = Rc::downgrade(&terminal);
        let created = Rc::new(RefCell::new(None));
        let captured = Rc::clone(&created);
        marker.on_dispose().try_subscribe(move |_| {
            let terminal = weak.upgrade().unwrap();
            terminal
                .options()
                .set("convertEol", OptionValue::Bool(true))
                .map_err(xterm_core::event::EmitError::callback)?;
            *captured.borrow_mut() = Some(
                terminal
                    .register_marker(-2)
                    .map_err(xterm_core::event::EmitError::callback)?,
            );
            Ok(())
        });
        terminal.write(&format!("\x1b[{command}")).unwrap();
        assert!(marker.is_disposed(), "{command}");
        assert!(created.borrow().is_some(), "{command}");
        assert_eq!(
            terminal.options().get("convertEol").unwrap(),
            OptionValue::Bool(true)
        );
    }
}

#[test]
fn marker_errors_retain_identity_and_stop_after_partial_mutation() {
    use std::error::Error;
    use xterm_core::event::EmitError;
    for command in ["2J", "L", "M", "S", "T", "3J"] {
        let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
        terminal
            .write("0\r\n1\r\n2\r\n3\r\n4\r\n5\r\n6\r\n7")
            .unwrap();
        let offset = match command {
            "S" => -2,
            "3J" => -5,
            _ => 0,
        };
        let marker = terminal.register_marker(offset).unwrap();
        let original = EmitError::callback(std::io::Error::other("marker disposal failure"));
        let captured = original.clone();
        marker
            .on_dispose()
            .try_subscribe(move |_| Err(captured.clone()));
        let error = terminal.write(&format!("\x1b[{command}")).unwrap_err();
        let mut source = &error as &dyn Error;
        let mut found = false;
        loop {
            if let Some(actual) = source.downcast_ref::<EmitError>() {
                found |= actual == &original;
            }
            match source.source() {
                Some(next) => source = next,
                None => break,
            }
        }
        assert!(found, "{command}: {error:?}");
        assert!(std::ptr::eq(
            source.downcast_ref::<std::io::Error>().unwrap(),
            original.callback_ref::<std::io::Error>().unwrap()
        ));
        assert!(marker.is_disposed(), "{command}");
        let active = terminal.with_state(|s| s.active_buffer()).unwrap();
        let b = active.borrow();
        let expected_length = match command {
            "2J" => 8,
            "3J" => 3,
            _ => 7,
        };
        assert_eq!(b.lines.len(), expected_length, "{command}");
        if command == "2J" {
            assert_eq!(b.lines.get(7).unwrap().borrow().get_trimmed_length(), 0);
            assert_eq!(b.lines.get(6).unwrap().borrow().get_trimmed_length(), 1);
        }
    }
}
