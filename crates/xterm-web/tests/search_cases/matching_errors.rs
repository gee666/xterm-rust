// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource};

type Read = dyn Fn() -> Result<SearchOptions, SearchAddonError>;
struct MatchingFailure(Rc<Read>);
impl SearchOptionsSource for MatchingFailure {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        (self.0)()
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        Ok(None)
    }
}

#[test]
fn matching_failure_follows_selection_clear_and_cache_init_without_live_borrows() {
    for reverse in [false, true] {
        let h = Harness::new(20, 3, None);
        h.write("foo FOO");
        h.addon.find_next(&term("foo"), None).unwrap();
        h.host.clock.advance(30_000).unwrap();
        assert_eq!(h.host.clock.pending(), 0);
        let events = Rc::new(RefCell::new(Vec::new()));
        let capture = Rc::clone(&events);
        h.addon
            .on_before_search()
            .subscribe(move |_| capture.borrow_mut().push("before"));
        let capture = Rc::clone(&events);
        h.addon
            .on_after_search()
            .subscribe(move |_| capture.borrow_mut().push("after"));
        let capture = Rc::clone(&events);
        h.addon
            .on_did_change_results()
            .subscribe(move |_| capture.borrow_mut().push("results"));
        let addon = h.addon.downgrade();
        let host = Rc::clone(&h.host);
        let request = Rc::new(MatchingFailure(Rc::new(move || {
            assert!(host.terminal.selection_position()?.is_none());
            assert_eq!(host.clock.pending(), 1);
            // A callback can select through the real services before throwing.
            let mut options = SearchOptions::default();
            options.matching.case_sensitive = Some(true);
            assert!(addon
                .upgrade()
                .unwrap()
                .find_next(&term("FOO"), Some(Rc::new(RefCell::new(options))))?);
            Err(SearchAddonError::InvalidCoordinates)
        })));
        let no_scroll = Cell::new(false);
        let result = if reverse {
            h.addon
                .find_previous_with_scroll(&term("foo"), Some(request), || {
                    no_scroll.set(true);
                    Ok(false)
                })
        } else {
            h.addon
                .find_next_with_scroll(&term("foo"), Some(request), || {
                    no_scroll.set(true);
                    Ok(false)
                })
        };
        assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
        assert!(!no_scroll.get());
        assert_eq!(h.selected_text(), term("FOO"));
        assert_eq!(*events.borrow(), ["before", "before", "after"]);
    }
}

#[test]
fn invalid_term_does_not_read_failing_matching_options_or_initialize_cache() {
    for reverse in [false, true] {
        let h = Harness::new(20, 3, None);
        h.write("foo");
        let request = Rc::new(MatchingFailure(Rc::new(|| {
            panic!("invalid term must short circuit matching access")
        })));
        let result = if reverse {
            h.addon.find_previous(&[], Some(request))
        } else {
            h.addon.find_next(&[], Some(request))
        };
        assert!(!result.unwrap());
        assert_eq!(h.host.clock.pending(), 0);
    }
}
