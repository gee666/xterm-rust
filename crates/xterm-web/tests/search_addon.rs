// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
mod search_support;
use search_support::{options, term, Element, Harness};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use xterm_addons::search::{
    DecorationManager, NumericHighlightLimit, SearchAddon, SearchAddonError, SearchHighlightLimit,
    SearchOptions, SearchOptionsHandle, SearchResultChange,
};
use xterm_core::{
    decoration::DecorationLayer,
    search::{SearchPosition, SearchResult, SearchSelection},
};

#[path = "search_cases/cache_queries.rs"]
mod cache_queries;
#[path = "search_cases/clocks.rs"]
mod clocks;
#[path = "search_cases/decorations.rs"]
mod decorations;
#[path = "search_cases/ranges.rs"]
mod ranges;
#[path = "search_cases/tracker.rs"]
mod tracker;

#[test]
fn use_before_activation_errors_without_search_events() {
    let addon = SearchAddon::default();
    let called = Rc::new(Cell::new(false));
    let capture = Rc::clone(&called);
    addon
        .on_before_search()
        .subscribe(move |_| capture.set(true));
    assert!(matches!(
        addon.find_next(&term("x"), None),
        Err(SearchAddonError::NotLoaded)
    ));
    assert!(!called.get());
}

#[test]
fn selections_decorations_result_indices_and_event_order_share_real_services() {
    let h = Harness::new(20, 3, None);
    h.write("hello hello");
    let events = Rc::new(RefCell::new(Vec::new()));
    let capture = Rc::clone(&events);
    h.addon
        .on_before_search()
        .subscribe(move |_| capture.borrow_mut().push("before".into()));
    let capture = Rc::clone(&events);
    h.addon.on_did_change_results().subscribe(move |result| {
        capture.borrow_mut().push(format!(
            "results:{}/{}",
            result.result_index, result.result_count
        ))
    });
    let capture = Rc::clone(&events);
    h.addon
        .on_after_search()
        .subscribe(move |_| capture.borrow_mut().push("after".into()));
    let request = options();
    assert!(h
        .addon
        .find_next(&term("hello"), Some(Rc::clone(&request)))
        .unwrap());
    assert_eq!(h.selected_text(), term("hello"));
    assert_eq!(
        h.selection(),
        Some(SearchSelection {
            start: SearchPosition { row: 0, col: 0 },
            end: SearchPosition { row: 0, col: 5 }
        })
    );
    assert_eq!(*events.borrow(), ["before", "results:0/2", "after"]);
    assert_eq!(h.decorations().len(), 3);
    events.borrow_mut().clear();
    assert!(h.addon.find_next(&term("hello"), Some(request)).unwrap());
    assert_eq!(h.selection().unwrap().start.col, 6);
    assert_eq!(*events.borrow(), ["before", "results:1/2", "after"]);
    assert_eq!(h.decorations().len(), 3);
    assert_eq!(
        h.decorations()
            .iter()
            .filter(|d| d.options().borrow().layer == Some(DecorationLayer::Top))
            .count(),
        1
    );
}

#[test]
fn original_wrapped_match_decoration_ranges_and_overview_rows() {
    let h = Harness::new(10, 5, None);
    h.write("0123456789abcde");
    assert!(h.addon.find_next(&term("9abc"), None).unwrap());
    assert_eq!(h.selected_text(), term("9abc"));
    let selected = h.selection().unwrap();
    let manager = DecorationManager::new(h.host.clone());
    let request = options().read().unwrap();
    manager
        .create_highlights(
            &[SearchResult {
                term: term("9abc"),
                row: selected.start.row,
                col: selected.start.col,
                size: 4,
            }],
            request.decorations.as_ref().unwrap(),
        )
        .unwrap();
    let decorations = h.decorations();
    assert_eq!(decorations.len(), 2);
    let first = decorations[0].options();
    let second = decorations[1].options();
    assert_eq!(
        (first.borrow().x, first.borrow().width),
        (Some(9.0), Some(1.0))
    );
    assert_eq!(
        (second.borrow().x, second.borrow().width),
        (Some(0.0), Some(3.0))
    );
    assert_eq!(
        decorations
            .iter()
            .filter(|d| d.options().borrow().overview_ruler_options.is_some())
            .count(),
        2
    );
    manager.clear_highlights().unwrap();
}

#[test]
fn original_only_one_overview_marker_per_buffer_line() {
    let h = Harness::new(10, 5, None);
    h.write("abcdefghij");
    let manager = DecorationManager::new(h.host.clone());
    manager
        .create_highlights(
            &[
                SearchResult {
                    term: term("a"),
                    row: 0,
                    col: 0,
                    size: 1,
                },
                SearchResult {
                    term: term("f"),
                    row: 0,
                    col: 5,
                    size: 1,
                },
            ],
            options().read().unwrap().decorations.as_ref().unwrap(),
        )
        .unwrap();
    let decorations = h.decorations();
    let ruler: Vec<_> = decorations
        .iter()
        .filter(|d| d.options().borrow().overview_ruler_options.is_some())
        .collect();
    assert_eq!(ruler.len(), 1);
    assert_eq!(ruler[0].options().borrow().x, Some(0.0));
    manager.clear_highlights().unwrap();
}

#[test]
fn clearing_decorations_does_not_clear_selection_despite_typing_comment() {
    let h = Harness::new(20, 3, None);
    h.write("hello hello");
    assert!(h.addon.find_next(&term("hello"), Some(options())).unwrap());
    let selection = h.selection();
    h.addon.clear_active_decoration().unwrap();
    assert_eq!(h.selection(), selection);
    assert_eq!(h.decorations().len(), 2);
    h.addon.clear_decorations(false).unwrap();
    assert_eq!(h.selection(), selection);
    assert!(h.decorations().is_empty());
    assert!(!h.addon.find_next(&[], Some(options())).unwrap());
    assert!(h.selection().is_none());
}

#[test]
fn active_styles_preserve_source_false_argument_and_border_once() {
    let h = Harness::new(20, 3, None);
    h.write("hello");
    h.addon.find_next(&term("hello"), Some(options())).unwrap();
    for decoration in h.decorations() {
        let element = Rc::new(Element::default());
        let opaque: Rc<dyn std::any::Any> = element.clone();
        decoration.fire_render(opaque.clone()).unwrap();
        assert_eq!(
            *element.classes.borrow(),
            ["xterm-find-result-decoration".to_owned()]
                .into_iter()
                .collect()
        );
        let border = if decoration.options().borrow().layer == Some(DecorationLayer::Top) {
            "#445566"
        } else {
            "#223344"
        };
        assert_eq!(
            element.outline.borrow().as_deref(),
            Some(format!("1px solid {border}").as_str())
        );
        *element.outline.borrow_mut() = Some("custom".into());
        decoration.fire_render(opaque).unwrap();
        assert_eq!(element.outline.borrow().as_deref(), Some("custom"));
    }
}

#[test]
fn fractional_and_nonfinite_limits_retain_slice_and_highlight_distinction() {
    for (limit, inactive, count) in [
        (0.0, 0, 0),
        (-1.0, 0, 0),
        (1.5, 2, 1),
        (f64::NAN, 3, 0),
        (f64::INFINITY, 3, 3),
    ] {
        let h = Harness::new(20, 3, Some(limit));
        h.write("a a a");
        let observed = Rc::new(Cell::new(None));
        let capture = Rc::clone(&observed);
        h.addon
            .on_did_change_results()
            .subscribe(move |event| capture.set(Some(*event)));
        assert!(h.addon.find_next(&term("a"), Some(options())).unwrap());
        assert_eq!(h.decorations().len(), inactive + 1);
        assert_eq!(observed.get().unwrap().result_count, count);
        assert_eq!(
            observed.get().unwrap().result_index,
            if count > 0 { 0 } else { -1 }
        );
    }
    for (limit, length, expected) in [
        (-1.0, 4, 3),
        (-1.9, 4, 3),
        (-0.5, 4, 0),
        (f64::NEG_INFINITY, 4, 0),
        (f64::INFINITY, 4, 4),
        (f64::NAN, 4, 0),
    ] {
        assert_eq!(
            NumericHighlightLimit(limit).end_index(length).unwrap(),
            expected
        );
    }
}

#[test]
fn on_before_search_can_mutate_the_same_options_object() {
    let h = Harness::new(20, 3, None);
    h.write("foo FOO");
    let request = Rc::new(RefCell::new(SearchOptions::default()));
    let capture = Rc::clone(&request);
    h.addon
        .on_before_search()
        .subscribe(move |_| capture.borrow_mut().matching.case_sensitive = Some(true));
    let handle: SearchOptionsHandle = request;
    assert!(h.addon.find_next(&term("FOO"), Some(handle)).unwrap());
    assert_eq!(h.selection().unwrap().start.col, 4);
}

#[test]
fn unchanged_term_option_change_preserves_source_highlight_order_bug() {
    let h = Harness::new(20, 3, None);
    h.write("ab a");
    h.addon.find_next(&term("a"), Some(options())).unwrap();
    let mut request = options().read().unwrap();
    request.matching.whole_word = Some(true);
    let observed = Rc::new(Cell::new(None));
    let capture = Rc::clone(&observed);
    h.addon
        .on_did_change_results()
        .subscribe(move |event| capture.set(Some(*event)));
    h.addon
        .find_next(&term("a"), Some(Rc::new(RefCell::new(request))))
        .unwrap();
    assert_eq!(h.selection().unwrap().start.col, 3);
    assert_eq!(
        observed.get(),
        Some(SearchResultChange {
            result_count: 2,
            result_index: 1
        })
    );
}

#[test]
fn clearing_and_result_events_are_reentrant_without_internal_borrows() {
    let h = Harness::new(20, 3, None);
    h.write("hello hello");
    let addon = h.addon.downgrade();
    h.addon.on_did_change_results().try_subscribe(move |_| {
        addon
            .upgrade()
            .unwrap()
            .clear_active_decoration()
            .map_err(xterm_core::event::EmitError::callback)
    });
    assert!(h.addon.find_next(&term("hello"), Some(options())).unwrap());
    assert_eq!(h.decorations().len(), 2);
    assert_eq!(h.selected_text(), term("hello"));
}
