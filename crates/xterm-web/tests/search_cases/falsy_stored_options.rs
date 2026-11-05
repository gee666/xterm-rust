// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource, WeakSearchAddon};

struct Request {
    truthy: bool,
    decorated: bool,
    replacement: Option<(WeakSearchAddon, Option<SearchOptionsHandle>)>,
    entered: Cell<bool>,
    comparisons: Cell<usize>,
}
impl SearchOptionsSource for Request {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn is_truthy(&self) -> bool {
        self.truthy
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        if self.decorated {
            options().read()
        } else {
            Ok(SearchOptions::default())
        }
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        if !self.entered.replace(true) {
            if let Some((addon, replacement)) = &self.replacement {
                assert!(addon
                    .upgrade()
                    .unwrap()
                    .find_next(&term("foo"), replacement.clone())?);
            }
        }
        Ok(self.read()?.decorations)
    }
    fn flags_changed(
        &self,
        _stored: &dyn Fn() -> Option<SearchOptionsHandle>,
    ) -> Result<bool, SearchAddonError> {
        self.comparisons.set(self.comparisons.get() + 1);
        Err(SearchAddonError::InvalidCoordinates)
    }
}
fn request(truthy: bool, decorated: bool) -> Request {
    Request {
        truthy,
        decorated,
        replacement: None,
        entered: Cell::new(false),
        comparisons: Cell::new(0),
    }
}
fn find(
    h: &Harness,
    reverse: bool,
    request: SearchOptionsHandle,
) -> Result<bool, SearchAddonError> {
    if reverse {
        h.addon.find_previous(&term("foo"), Some(request))
    } else {
        h.addon.find_next(&term("foo"), Some(request))
    }
}

#[test]
fn falsy_stored_request_updates_highlights_without_comparing_flags() {
    for reverse in [false, true] {
        let h = Harness::new(20, 3, None);
        h.write("foo FOO foo");
        assert!(h.addon.find_next(&term("foo"), None).unwrap());
        let request = Rc::new(request(false, true));
        assert!(find(&h, reverse, request.clone()).unwrap());
        assert_eq!(request.comparisons.get(), 0);
        assert_eq!(h.decorations().len(), 4);
        assert_eq!(h.selected_text(), term(if reverse { "foo" } else { "FOO" }));
    }
}

#[test]
fn nested_decoration_search_replaces_stored_request_before_comparison() {
    for reverse in [false, true] {
        for replacement in [
            None,
            Some(Rc::new(request(false, false)) as SearchOptionsHandle),
        ] {
            let h = Harness::new(20, 3, None);
            h.write("foo FOO foo");
            assert!(h.addon.find_next(&term("foo"), None).unwrap());
            let mut outer = request(true, true);
            outer.replacement = Some((h.addon.downgrade(), replacement));
            let outer = Rc::new(outer);
            assert!(find(&h, reverse, outer.clone()).unwrap());
            assert!(outer.entered.get());
            assert_eq!(outer.comparisons.get(), 0);
            assert_eq!(h.decorations().len(), 4);
        }
    }
}

#[test]
fn falsy_incoming_request_skips_comparison_after_truthy_nested_replacement() {
    for reverse in [false, true] {
        let h = Harness::new(20, 3, None);
        h.write("foo FOO foo");
        h.addon.find_next(&term("foo"), None).unwrap();
        let mut outer = request(false, true);
        outer.replacement = Some((h.addon.downgrade(), Some(Rc::new(request(true, false)))));
        let outer = Rc::new(outer);
        assert!(find(&h, reverse, outer.clone()).unwrap());
        assert_eq!(outer.comparisons.get(), 0);
        // No highlight update: the outer selection creates only its active decoration.
        assert_eq!(h.decorations().len(), 1);
    }
}

#[test]
fn truthy_nested_request_keeps_comparison_failure_and_nested_selection() {
    for reverse in [false, true] {
        let h = Harness::new(20, 3, None);
        h.write("foo FOO foo");
        h.addon.find_next(&term("foo"), None).unwrap();
        let mut outer = request(true, true);
        outer.replacement = Some((h.addon.downgrade(), Some(Rc::new(request(true, false)))));
        let outer = Rc::new(outer);
        assert!(matches!(
            find(&h, reverse, outer.clone()),
            Err(SearchAddonError::InvalidCoordinates)
        ));
        assert_eq!(outer.comparisons.get(), 1);
        assert_eq!(h.selected_text(), term("FOO"));
        assert!(h.decorations().is_empty());
    }
}
