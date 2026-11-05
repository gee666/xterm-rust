// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource};

struct Probe {
    comparisons: Cell<usize>,
    fail: bool,
}
impl SearchOptionsSource for Probe {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        panic!("invalid terms must not enter matching")
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        Ok(options().read()?.decorations)
    }
    fn flags_changed(
        &self,
        _stored: &dyn Fn() -> Option<SearchOptionsHandle>,
    ) -> Result<bool, SearchAddonError> {
        self.comparisons.set(self.comparisons.get() + 1);
        if self.fail {
            Err(SearchAddonError::InvalidCoordinates)
        } else {
            Ok(false)
        }
    }
}
fn probe(h: &Harness, reverse: bool, expected: usize, fail: bool) {
    let request = Rc::new(Probe {
        comparisons: Cell::new(0),
        fail,
    });
    let result = if reverse {
        h.addon.find_previous(&[], Some(request.clone()))
    } else {
        h.addon.find_next(&[], Some(request.clone()))
    };
    if fail && expected > 0 {
        assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
    } else {
        assert!(!result.unwrap());
    }
    assert_eq!(request.comparisons.get(), expected);
}
fn schedule(h: &Harness, resize: bool) {
    h.write("foo foo");
    assert!(h.addon.find_next(&term("foo"), Some(options())).unwrap());
    if resize {
        h.host.terminal.native.resize(21, 3).unwrap();
    } else {
        h.write("\x1b[0m");
    }
}

#[test]
fn queued_refresh_after_cache_clear_does_not_cache_an_empty_string() {
    for resize in [false, true] {
        for reverse in [false, true] {
            for fail in [false, true] {
                let h = Harness::new(20, 3, None);
                schedule(&h, resize);
                h.addon.clear_decorations(false).unwrap();
                let events = Rc::new(Cell::new(0));
                let capture = Rc::clone(&events);
                h.addon.on_after_search().subscribe(move |_| {
                    capture.set(capture.get() + 1);
                });
                h.host.clock.advance(200).unwrap();
                assert_eq!(events.get(), 1);
                assert!(h.selection().is_none());
                assert!(h.decorations().is_empty());
                probe(&h, reverse, 0, fail);
            }
        }
    }
}

#[test]
fn queued_refresh_keeps_explicit_empty_text_distinct_from_absence() {
    for resize in [false, true] {
        for reverse in [false, true] {
            for fail in [false, true] {
                let h = Harness::new(20, 3, None);
                schedule(&h, resize);
                assert!(!h.addon.find_next(&[], None).unwrap());
                h.host.clock.advance(200).unwrap();
                probe(&h, reverse, 1, fail);
            }
        }
    }
}

#[test]
fn refresh_captures_absence_before_reentrant_option_copy() {
    for resize in [false, true] {
        let h = Harness::new(20, 3, None);
        h.write("foo foo");
        let addon = h.addon.downgrade();
        let request = Rc::new(super::refresh::Request {
            options: options().read().unwrap(),
            refresh: Rc::new(move || {
                let addon = addon.upgrade().unwrap();
                assert!(!addon.find_next(&[], Some(options()))?);
                Ok(options())
            }),
        });
        h.addon.find_next(&term("foo"), Some(request)).unwrap();
        if resize {
            h.host.terminal.native.resize(21, 3).unwrap();
        } else {
            h.write("\x1b[0m");
        }
        h.addon.clear_decorations(false).unwrap();
        h.host.clock.advance(200).unwrap();
        probe(&h, false, 0, true);
    }
}
