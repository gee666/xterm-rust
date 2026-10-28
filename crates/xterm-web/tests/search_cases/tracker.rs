// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::SearchResultTracker;
use xterm_core::event::EmitError;

fn found(term_value: &str, col: isize) -> SearchResult {
    SearchResult {
        term: term(term_value),
        row: 0,
        col,
        size: 1,
    }
}

#[test]
fn tracker_preserves_slice_order_and_ignores_term_in_identity() {
    let tracker = SearchResultTracker::default();
    tracker.update_results(vec![found("a", 0), found("b", 2), found("c", 4)], 2.9);
    assert_eq!(tracker.results(), [found("a", 0), found("b", 2)]);
    assert_eq!(tracker.find_result_index(&found("different", 2)), 1);
    assert_eq!(tracker.find_result_index(&found("a", 4)), -1);
    let observed = Rc::new(Cell::new(None));
    let capture = Rc::clone(&observed);
    tracker
        .on_did_change_results()
        .subscribe(move |result| capture.set(Some(*result)));
    tracker.fire_results_changed(false).unwrap();
    assert!(observed.get().is_none());
    tracker.fire_results_changed(true).unwrap();
    assert_eq!(
        observed.get(),
        Some(SearchResultChange {
            result_index: -1,
            result_count: 2
        })
    );
    tracker.reset().unwrap();
    assert!(tracker.results().is_empty());
}

#[test]
fn selected_decoration_disposes_its_marker_and_subscription_once() {
    let h = Harness::new(20, 3, None);
    h.write("a");
    let manager = DecorationManager::new(h.host.clone());
    let active = manager
        .create_active(
            found("a", 0),
            options().read().unwrap().decorations.as_ref().unwrap(),
        )
        .unwrap()
        .unwrap();
    let decoration = active.decorations()[0].clone();
    let marker = decoration.marker();
    let disposed = Rc::new(Cell::new(0));
    let capture = Rc::clone(&disposed);
    marker
        .on_dispose()
        .subscribe(move |_| capture.set(capture.get() + 1));
    let tracker = SearchResultTracker::default();
    tracker.set_selected(Some(active));
    tracker.update_results(vec![found("a", 0)], 10.0);
    let result = Rc::new(Cell::new(None));
    let capture = Rc::clone(&result);
    tracker
        .on_did_change_results()
        .subscribe(move |event| capture.set(Some(*event)));
    tracker.fire_results_changed(true).unwrap();
    assert_eq!(
        result.get(),
        Some(SearchResultChange {
            result_index: 0,
            result_count: 1
        })
    );
    tracker.clear_selected().unwrap();
    tracker.clear_selected().unwrap();
    assert_eq!(disposed.get(), 1);
    assert!(marker.is_disposed());
    assert!(decoration.is_disposed());
    assert!(h.decorations().is_empty());
}

#[test]
fn selected_failure_retains_original_error_and_tracker_until_retry() {
    let h = Harness::new(20, 3, None);
    h.write("a");
    let manager = DecorationManager::new(h.host.clone());
    let active = manager
        .create_active(
            found("a", 0),
            options().read().unwrap().decorations.as_ref().unwrap(),
        )
        .unwrap()
        .unwrap();
    let decoration = active.decorations()[0].clone();
    let original = EmitError::callback(std::io::Error::other("decoration callback sentinel"));
    let sentinel = original.clone();
    let failure = decoration
        .on_dispose()
        .try_subscribe(move |_| Err(sentinel.clone()));
    let tracker = SearchResultTracker::default();
    tracker.set_selected(Some(active));
    tracker.update_results(vec![found("a", 0)], 10.0);
    assert_eq!(tracker.reset(), Err(original));
    assert!(tracker.selected().is_some());
    assert_eq!(tracker.results().len(), 1);
    failure.dispose();
    tracker.reset().unwrap();
    assert!(tracker.selected().is_none());
    assert!(tracker.results().is_empty());
}

#[test]
fn selected_replacement_inside_disposal_is_cleared_like_source_assignment() {
    let h = Harness::new(20, 3, None);
    h.write("a b");
    let manager = DecorationManager::new(h.host.clone());
    let colors = options().read().unwrap().decorations.unwrap();
    let first = manager
        .create_active(found("a", 0), &colors)
        .unwrap()
        .unwrap();
    let second = manager
        .create_active(found("b", 2), &colors)
        .unwrap()
        .unwrap();
    let tracker = Rc::new(SearchResultTracker::default());
    tracker.set_selected(Some(first.clone()));
    let weak = Rc::downgrade(&tracker);
    let replacement = Rc::clone(&second);
    first.decorations()[0].on_dispose().subscribe(move |_| {
        weak.upgrade()
            .unwrap()
            .set_selected(Some(Rc::clone(&replacement)))
    });
    tracker.clear_selected().unwrap();
    assert!(tracker.selected().is_none());
    assert!(!second.decorations()[0].is_disposed());
    second.dispose().unwrap();
}

#[test]
fn result_event_can_reenter_tracker_reset_without_a_live_results_borrow() {
    let tracker = Rc::new(SearchResultTracker::default());
    tracker.update_results(vec![found("a", 0)], 10.0);
    let weak = Rc::downgrade(&tracker);
    tracker.on_did_change_results().try_subscribe(move |event| {
        assert_eq!(event.result_count, 1);
        weak.upgrade().unwrap().reset()
    });
    tracker.fire_results_changed(true).unwrap();
    assert!(tracker.results().is_empty());
}
