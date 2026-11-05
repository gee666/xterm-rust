// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource};

type Refresh = dyn Fn() -> Result<SearchOptionsHandle, SearchAddonError>;
pub(super) struct Request {
    pub(super) options: SearchOptions,
    pub(super) refresh: Rc<Refresh>,
}
impl SearchOptionsSource for Request {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        Ok(self.options.clone())
    }
    fn for_refresh(&self) -> Result<SearchOptionsHandle, SearchAddonError> {
        (self.refresh)()
    }
}
fn decorated() -> SearchDecorationOptionsHandle {
    options().read().unwrap().decorations.unwrap()
}

#[test]
fn refresh_copy_runs_before_events_with_cleared_cache_and_no_live_state_borrow() {
    let h = Harness::new(20, 3, None);
    h.write("foo foo");
    let log = Rc::new(RefCell::new(Vec::new()));
    let capture = Rc::clone(&log);
    let addon = h.addon.downgrade();
    let host = Rc::clone(&h.host);
    let request = Rc::new(Request {
        options: SearchOptions {
            decorations: Some(decorated()),
            ..SearchOptions::default()
        },
        refresh: Rc::new(move || {
            capture.borrow_mut().push("copy");
            let addon = addon.upgrade().unwrap();
            // A cached same-term search would select the second occurrence.
            assert!(addon.find_next(&term("foo"), None)?);
            assert_eq!(host.terminal.selection_position()?.unwrap().start.col, 0);
            Ok(options())
        }),
    });
    h.addon.find_next(&term("foo"), Some(request)).unwrap();
    h.write("\x1b[0m");
    let capture = Rc::clone(&log);
    h.addon
        .on_before_search()
        .subscribe(move |_| capture.borrow_mut().push("before"));
    let capture = Rc::clone(&log);
    h.addon
        .on_after_search()
        .subscribe(move |_| capture.borrow_mut().push("after"));
    h.host.clock.advance(200).unwrap();
    assert_eq!(
        *log.borrow(),
        ["copy", "before", "after", "before", "after"]
    );
}

#[test]
fn refresh_copy_failure_keeps_cache_cleared_and_does_not_start_search() {
    let h = Harness::new(20, 3, None);
    h.write("foo foo");
    let request = Rc::new(Request {
        options: SearchOptions {
            decorations: Some(decorated()),
            ..SearchOptions::default()
        },
        refresh: Rc::new(|| Err(SearchAddonError::InvalidCoordinates)),
    });
    h.addon.find_next(&term("foo"), Some(request)).unwrap();
    h.write("\x1b[0m");
    let count = Rc::new(Cell::new(0));
    let capture = Rc::clone(&count);
    h.addon
        .on_before_search()
        .subscribe(move |_| capture.set(capture.get() + 1));
    assert!(matches!(
        h.host.clock.advance(200),
        Err(SearchAddonError::InvalidCoordinates)
    ));
    assert_eq!(count.get(), 0);
    assert!(h.addon.find_next(&term("foo"), None).unwrap());
    assert_eq!(h.selection().unwrap().start.col, 0);
}

#[test]
fn native_refresh_copy_is_incremental_and_independent_of_original_mutation() {
    let original = Rc::new(RefCell::new(SearchOptions {
        decorations: Some(decorated()),
        matching: xterm_core::search::SearchOptions {
            case_sensitive: Some(true),
            incremental: Some(false),
            ..Default::default()
        },
    }));
    let copy = original.for_refresh().unwrap();
    original.borrow_mut().matching.case_sensitive = Some(false);
    original.borrow_mut().decorations = None;
    let copied = copy.read().unwrap();
    assert_eq!(copied.matching.case_sensitive, Some(true));
    assert_eq!(copied.matching.incremental, Some(true));
    assert!(copied.decorations.is_some());
}
