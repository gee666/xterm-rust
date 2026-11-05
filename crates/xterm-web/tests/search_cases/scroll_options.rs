// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;

#[test]
fn no_scroll_is_read_after_matching_and_can_reenter_without_state_borrows() {
    let h = Harness::new(20, 3, None);
    h.write("a a\r\nrow1\r\nrow2\r\nrow3\r\nrow4");
    let reads = Cell::new(0);
    assert!(h
        .addon
        .find_next_with_scroll(&term("a"), None, || {
            reads.set(reads.get() + 1);
            assert!(h.selection().is_none());
            assert!(h.addon.find_next(&term("a"), None)?);
            Ok(true)
        })
        .unwrap());
    assert_eq!(reads.get(), 1);
    assert_eq!(h.selected_text(), term("a"));
}

#[test]
fn internal_getter_errors_skip_selection_and_after_event() {
    let h = Harness::new(20, 3, None);
    h.write("a a");
    h.addon.find_next(&term("a"), None).unwrap();
    let after = Rc::new(Cell::new(false));
    let capture = Rc::clone(&after);
    h.addon.on_after_search().subscribe(move |_| capture.set(true));
    let result = h.addon.find_previous_with_scroll(&term("a"), None, || {
        assert!(h.selection().is_none());
        Err(SearchAddonError::InvalidCoordinates)
    });
    assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
    assert!(h.selection().is_none());
    assert!(!after.get());
}

#[test]
fn empty_and_unloaded_searches_skip_internal_getters_but_no_match_reads_them() {
    let h = Harness::new(20, 3, None);
    let reads = Cell::new(0);
    let getter = || {
        reads.set(reads.get() + 1);
        Ok(false)
    };
    assert!(!h.addon.find_next_with_scroll(&[], None, getter).unwrap());
    assert_eq!(reads.get(), 0);
    let addon = SearchAddon::default();
    assert!(matches!(
        addon.find_previous_with_scroll(&term("a"), None, getter),
        Err(SearchAddonError::NotLoaded)
    ));
    assert_eq!(reads.get(), 0);
    assert!(!h
        .addon
        .find_previous_with_scroll(&term("missing"), None, getter)
        .unwrap());
    assert_eq!(reads.get(), 1);
}
