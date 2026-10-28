// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_core::{event::EmitError, terminal::CoreTerminal};
fn rows(t: &CoreTerminal) -> Vec<Vec<u16>> {
    let b = t.state().active_buffer();
    let b = b.borrow();
    (0..b.lines.len())
        .map(|i| {
            b.lines
                .get(i)
                .unwrap()
                .borrow()
                .translate_to_string(true, None, None, None)
        })
        .collect()
}
#[test]
fn insert_delete_lines_allow_buffer_observers_to_mutate_cursor() {
    for command in ['L', 'M'] {
        let mut t = CoreTerminal::new(4, 4, 0).unwrap();
        t.write("a\r\nb\r\nc\r\nd\x1b[2;4H").unwrap();
        let active = t.state().active_buffer();
        let lines = active.borrow().lines.share();
        let observed = Rc::new(RefCell::new(Vec::new()));
        let captured = observed.clone();
        let weak = Rc::downgrade(&active);
        lines
            .on_delete_emitter
            .event()
            .try_subscribe(move |change| {
                let active = weak.upgrade().unwrap();
                let mut b = active.try_borrow_mut().map_err(EmitError::callback)?;
                captured
                    .borrow_mut()
                    .push((change.index, change.amount, b.lines.len(), b.x, b.y));
                b.x = 2;
                Ok(())
            });
        t.write(&format!("\x1b[2{command}")).unwrap();
        let index = if command == 'L' { 3 } else { 1 };
        assert_eq!(
            *observed.borrow(),
            [(index, 1, 3, 3, 1), (index, 1, 3, 2, 1)]
        );
        assert_eq!(active.borrow().x, 0);
    }
}
#[test]
fn scroll_up_reads_live_margins_after_delete_observers() {
    let mut t = CoreTerminal::new(4, 4, 0).unwrap();
    t.write("a\r\nb\r\nc\r\nd\x1b[H").unwrap();
    let active = t.state().active_buffer();
    let lines = active.borrow().lines.share();
    let weak = Rc::downgrade(&active);
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    lines.on_delete_emitter.event().try_subscribe(move |_| {
        let active = weak.upgrade().unwrap();
        let mut b = active.try_borrow_mut().map_err(EmitError::callback)?;
        b.scroll_top = 1;
        b.scroll_bottom = 2;
        count.set(count.get() + 1);
        Ok(())
    });
    t.write("\x1b[2S").unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(
        rows(&t),
        ["b", "", "", "d"].map(|s| s.encode_utf16().collect::<Vec<_>>())
    );
}
#[test]
fn deletion_failure_preserves_exact_callback_error_and_partial_state() {
    use std::error::Error;
    let mut t = CoreTerminal::new(4, 4, 0).unwrap();
    let active = t.state().active_buffer();
    let lines = active.borrow().lines.share();
    let original = EmitError::callback(std::io::Error::other("deletion callback failed"));
    let captured = original.clone();
    lines
        .on_delete_emitter
        .event()
        .try_subscribe(move |_| Err(captured.clone()));
    let error = t.write("\x1b[L").unwrap_err();
    // Parse/dispatch context is allowed; it must retain the same callback Rc,
    // not merely another error with the same message or type.
    let mut found_original = false;
    let mut source = &error as &dyn Error;
    loop {
        if let Some(actual) = source.downcast_ref::<EmitError>() {
            found_original |= actual == &original;
        }
        match source.source() {
            Some(next) => source = next,
            None => break,
        }
    }
    assert!(
        found_original,
        "lost exact callback error identity: {error:?}"
    );
    let expected = original.callback_ref::<std::io::Error>().unwrap();
    let actual = source.downcast_ref::<std::io::Error>().unwrap();
    assert!(
        std::ptr::eq(actual, expected),
        "replaced original callback cause"
    );
    assert_eq!(actual.to_string(), "deletion callback failed");
    assert_eq!(active.borrow().lines.len(), 3);
}
