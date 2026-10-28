// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use crate::accessibility_support::Fixture;
use std::{cell::Cell, rc::Rc};
use wasm_bindgen_test::*;
use xterm_web::{accessibility::AccessibilityEvent as Event, dom_listener::DomListener};

#[wasm_bindgen_test]
fn boundary_navigation_rotates_nodes_scrolls_and_cancels_owned_timer_on_dispose() {
    let f = Fixture::new();
    let first = f.row(0);
    let removed = f.row(2);
    f.row(1).focus().unwrap();
    first.focus().unwrap();
    assert_eq!(*f.host.scrolls.borrow(), [-1]);
    assert_eq!(f.host.offset.get(), 1);
    assert_ne!(f.row(0), first);
    assert_eq!(f.row(1), first);
    assert!(removed.parent_node().is_none());
    f.render();
    let bottom = f.row(2);
    f.row(1).focus().unwrap();
    bottom.focus().unwrap();
    assert_eq!(*f.host.scrolls.borrow(), [-1, 1]);
    assert_eq!(f.host.offset.get(), 2);
    assert_eq!(f.row(1), bottom);
    assert_ne!(f.row(2), bottom);
    assert_eq!(f.tree().children().length(), 3);
    assert_eq!(f.clock.pending(), 1);
    f.manager.dispose().unwrap();
    assert_eq!(f.clock.pending(), 0);
}
#[wasm_bindgen_test]
fn outside_related_target_and_buffer_limits_do_not_scroll() {
    let f = Fixture::new();
    f.before.focus().unwrap();
    f.row(0).focus().unwrap();
    f.after.focus().unwrap();
    f.row(2).focus().unwrap();
    assert!(f.host.scrolls.borrow().is_empty());
    f.host.offset.set(0);
    f.host.emit(Event::Scroll).unwrap();
    f.clock.advance(1000.0).unwrap();
    f.row(1).focus().unwrap();
    f.row(0).focus().unwrap();
    f.host.offset.set(3);
    f.host.emit(Event::Scroll).unwrap();
    f.clock.advance(1000.0).unwrap();
    f.row(1).focus().unwrap();
    f.row(2).focus().unwrap();
    assert!(f.host.scrolls.borrow().is_empty());
}
#[wasm_bindgen_test]
fn resize_keeps_top_listener_order_then_rebinds_old_row_as_bottom() {
    let f = Fixture::new();
    let original = f.row(0);
    let calls = Rc::new(Cell::new(0));
    let count = Rc::clone(&calls);
    let _external = DomListener::new_reentrant(original.as_ref(), "focus", false, move |_| {
        count.set(count.get() + 1)
    })
    .unwrap();
    f.host.rows.set(4);
    f.host.emit(Event::Resize(4)).unwrap();
    f.render();
    f.row(1).focus().unwrap();
    original.focus().unwrap();
    assert_eq!(
        calls.get(),
        0,
        "retained top listener must stop the later external listener"
    );
    assert_eq!(*f.host.scrolls.borrow(), [-1]);
    f.host.rows.set(2);
    f.host.emit(Event::Resize(2)).unwrap();
    f.render();
    assert_eq!(f.row(1), original);
    f.before.focus().unwrap();
    f.row(0).focus().unwrap();
    original.focus().unwrap();
    assert_eq!(
        calls.get(),
        1,
        "new bottom listener was added after the existing external listener"
    );
    assert_eq!(*f.host.scrolls.borrow(), [-1, 1]);
}
#[wasm_bindgen_test]
fn one_row_viewport_has_no_adjacent_boundary_and_does_not_panic_or_scroll() {
    let f = Fixture::new();
    f.host.rows.set(1);
    f.host.emit(Event::Resize(1)).unwrap();
    f.render();
    f.before.focus().unwrap();
    f.row(0).focus().unwrap();
    assert_eq!(f.tree().children().length(), 1);
    assert!(f.host.scrolls.borrow().is_empty());
}
