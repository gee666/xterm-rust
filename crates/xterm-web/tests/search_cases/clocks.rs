// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;

#[test]
fn parsed_writes_refresh_at_200ms_and_replace_pending_refresh() {
    let h = Harness::new(20, 5, None);
    h.write("foo");
    h.addon.find_next(&term("foo"), Some(options())).unwrap();
    let observed = Rc::new(RefCell::new(Vec::new()));
    let capture = Rc::clone(&observed);
    h.addon
        .on_did_change_results()
        .subscribe(move |result| capture.borrow_mut().push(*result));
    h.write("\r\nfoo");
    h.host.clock.advance(100).unwrap();
    h.write("\r\nfoo");
    h.host.clock.advance(299).unwrap();
    assert!(observed.borrow().is_empty());
    let viewport = h
        .host
        .terminal
        .native
        .with_state(|state| state.active_buffer().borrow().ydisp)
        .unwrap();
    h.host.clock.advance(300).unwrap();
    assert_eq!(
        *observed.borrow(),
        [SearchResultChange {
            result_index: 0,
            result_count: 3
        }]
    );
    assert_eq!(
        h.host
            .terminal
            .native
            .with_state(|state| state.active_buffer().borrow().ydisp)
            .unwrap(),
        viewport
    );
    assert_eq!(h.host.clock.pending(), 1);
}

#[test]
fn cache_ttl_refreshes_last_access_without_replacing_every_timer() {
    let h = Harness::new(20, 3, None);
    h.write("foo");
    h.addon.find_next(&term("foo"), None).unwrap();
    assert_eq!(h.host.clock.pending(), 1);
    h.host.clock.advance(14_999).unwrap();
    assert_eq!(h.host.clock.pending(), 1);
    h.addon.find_next(&term("foo"), None).unwrap();
    assert_eq!(h.host.clock.pending(), 1);
    h.host.clock.advance(15_000).unwrap();
    assert_eq!(h.host.clock.pending(), 1);
    h.host.clock.advance(29_998).unwrap();
    assert_eq!(h.host.clock.pending(), 1);
    h.host.clock.advance(29_999).unwrap();
    assert_eq!(h.host.clock.pending(), 0);
    h.addon.find_next(&term("foo"), None).unwrap();
    assert_eq!(h.host.clock.pending(), 1);
}

#[test]
fn linefeed_cursor_and_resize_each_cancel_the_live_cache_timer() {
    for event in ["linefeed", "cursor", "resize"] {
        let h = Harness::new(20, 3, None);
        h.write("foo");
        h.addon.find_next(&term("foo"), None).unwrap();
        assert_eq!(h.host.clock.pending(), 1);
        match event {
            "linefeed" => h.write("\n"),
            "cursor" => h.write("x"),
            "resize" => h.host.terminal.native.resize(21, 3).unwrap(),
            _ => unreachable!(),
        }
        assert_eq!(h.host.clock.pending(), 0, "{event}");
    }
}

#[test]
fn disposal_cancels_debounce_and_cache_and_removes_actual_decorations() {
    let h = Harness::new(20, 3, None);
    h.write("foo");
    h.addon.find_next(&term("foo"), Some(options())).unwrap();
    h.write("\r\nfoo");
    assert_eq!(h.host.clock.pending(), 1);
    let after = Rc::new(Cell::new(0));
    let capture = Rc::clone(&after);
    h.addon
        .on_after_search()
        .subscribe(move |_| capture.set(capture.get() + 1));
    h.addon.dispose().unwrap();
    h.addon.dispose().unwrap();
    assert_eq!(h.host.clock.pending(), 0);
    assert!(h.decorations().is_empty());
    h.write("\r\nfoo");
    h.host.clock.advance(1000).unwrap();
    assert_eq!(after.get(), 0);
    assert_eq!(h.host.clock.pending(), 0);
}

#[test]
fn disposal_from_result_callback_does_not_retain_a_mutable_borrow() {
    let h = Harness::new(20, 3, None);
    h.write("foo");
    let addon = h.addon.downgrade();
    h.addon.on_did_change_results().try_subscribe(move |_| {
        addon
            .upgrade()
            .unwrap()
            .dispose()
            .map_err(xterm_core::event::EmitError::callback)
    });
    assert!(h.addon.find_next(&term("foo"), Some(options())).unwrap());
    assert!(h.decorations().is_empty());
    assert_eq!(h.host.clock.pending(), 0);
}
