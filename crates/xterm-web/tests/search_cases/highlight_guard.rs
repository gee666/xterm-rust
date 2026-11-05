// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use std::{any::Any, borrow::Cow};
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource, SearchTermSource};

type Read = dyn Fn() -> Result<bool, SearchAddonError>;
struct RawTerm {
    validations: Cell<usize>,
    guard: Rc<Read>,
    text: Rc<Read>,
}
impl SearchTermSource for RawTerm {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        let count = self.validations.get();
        self.validations.set(count + 1);
        Ok(count == 0)
    }
    fn is_valid_for_engine(&self) -> Result<bool, SearchAddonError> {
        (self.guard)()
    }
    fn is_truthy(&self) -> bool {
        true
    }
    fn equals(&self, _: &dyn SearchTermSource) -> bool {
        false
    }
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError> {
        (self.text)()?;
        Ok(Cow::Owned(term("foo")))
    }
}
struct Request {
    flags: Rc<Cell<usize>>,
    fail: bool,
}
impl SearchOptionsSource for Request {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        options().read()
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        Ok(self.read()?.decorations)
    }
    fn matching_options(&self) -> Result<xterm_core::search::SearchOptions, SearchAddonError> {
        self.flags.set(self.flags.get() + 1);
        if self.fail {
            return Err(SearchAddonError::MissingOptions);
        }
        Ok(Default::default())
    }
}
fn seed(h: &Harness) {
    h.write("foo foo foo");
    assert!(h.addon.find_next(&term("foo"), Some(options())).unwrap());
    h.host.clock.advance(30_000).unwrap();
    assert_eq!(h.decorations().len(), 4);
    assert_eq!(h.host.clock.pending(), 0);
}
fn no_text() -> Rc<Read> {
    Rc::new(|| panic!("highlight guard cutoff must skip matching text"))
}

#[test]
fn highlight_guard_runs_after_decoration_clear_before_cache_flags_and_text() {
    for reverse in [false, true] {
        for throwing in [false, true] {
            for reactivate in [false, true] {
                let h = Harness::new(20, 3, None);
                let other = Harness::new(20, 3, None);
                seed(&h);
                seed(&other);
                let host = Rc::clone(&h.host);
                let next = Rc::clone(&other.host);
                let weak = h.addon.downgrade();
                let calls = Rc::new(Cell::new(0));
                let capture = Rc::clone(&calls);
                let raw = Rc::new(RawTerm {
                    validations: Cell::new(0),
                    guard: Rc::new(move || {
                        capture.set(capture.get() + 1);
                        assert!(host.terminal.selection_position()?.is_some());
                        assert_eq!(host.terminal.decorations.decorations().unwrap().count(), 0);
                        assert_eq!(host.clock.pending(), 0);
                        if reactivate {
                            weak.upgrade().unwrap().activate(next.clone())?;
                        }
                        if throwing {
                            Err(SearchAddonError::InvalidCoordinates)
                        } else {
                            Ok(false)
                        }
                    }),
                    text: no_text(),
                });
                let flags = Rc::new(Cell::new(0));
                let request = Rc::new(Request {
                    flags: Rc::clone(&flags),
                    fail: false,
                });
                let result =
                    h.addon
                        .find_term_with_scroll(raw.clone(), Some(request), reverse, || {
                            panic!("later addon-invalid path skips noScroll")
                        });
                if throwing {
                    assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
                    assert!(h.selection().is_some());
                    assert!(other.selection().is_some());
                    assert_eq!(raw.validations.get(), 1);
                } else {
                    assert!(!result.unwrap());
                    assert!(h.selection().is_none());
                    assert_eq!(other.selection().is_none(), reactivate);
                    assert_eq!(raw.validations.get(), 2);
                }
                assert_eq!(calls.get(), 1);
                assert_eq!(flags.get(), 0);
                assert_eq!(h.host.clock.pending(), 0);
            }
        }
    }
}

#[test]
fn highlight_guard_repeats_after_each_found_match_and_preserves_throw_cutoff() {
    for reverse in [false, true] {
        for stop in [2, 3] {
            for throwing in [false, true] {
                let h = Harness::new(20, 3, None);
                seed(&h);
                let calls = Rc::new(Cell::new(0));
                let capture = Rc::clone(&calls);
                let texts = Rc::new(Cell::new(0));
                let text_capture = Rc::clone(&texts);
                let raw = Rc::new(RawTerm {
                    validations: Cell::new(0),
                    guard: Rc::new(move || {
                        capture.set(capture.get() + 1);
                        if capture.get() == stop {
                            if throwing {
                                Err(SearchAddonError::InvalidCoordinates)
                            } else {
                                Ok(false)
                            }
                        } else {
                            Ok(true)
                        }
                    }),
                    text: Rc::new(move || {
                        text_capture.set(text_capture.get() + 1);
                        Ok(true)
                    }),
                });
                let flags = Rc::new(Cell::new(0));
                let request = Rc::new(Request {
                    flags: Rc::clone(&flags),
                    fail: false,
                });
                let result = h
                    .addon
                    .find_term_with_scroll(raw, Some(request), reverse, || {
                        panic!("later invalid term skips noScroll")
                    });
                if throwing {
                    assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
                    assert!(h.selection().is_some());
                } else {
                    assert!(!result.unwrap());
                    assert!(h.selection().is_none());
                }
                assert_eq!(calls.get(), stop);
                assert_eq!(flags.get(), stop - 1);
                assert_eq!(texts.get(), stop - 1);
                assert_eq!(h.host.clock.pending(), 1);
                assert!(h.decorations().is_empty());
            }
        }
    }
}

#[test]
fn highlight_nonempty_guard_initializes_cache_before_option_error() {
    for reverse in [false, true] {
        let h = Harness::new(20, 3, None);
        seed(&h);
        let flags = Rc::new(Cell::new(0));
        let raw = Rc::new(RawTerm {
            validations: Cell::new(0),
            guard: Rc::new(|| Ok(true)),
            text: no_text(),
        });
        let request = Rc::new(Request {
            flags: Rc::clone(&flags),
            fail: true,
        });
        assert!(matches!(
            h.addon
                .find_term_with_scroll(raw, Some(request), reverse, || {
                    panic!("matching options failure skips noScroll")
                }),
            Err(SearchAddonError::MissingOptions)
        ));
        assert_eq!(flags.get(), 1);
        assert_eq!(h.host.clock.pending(), 1);
        assert!(h.selection().is_some());
        assert!(h.decorations().is_empty());
    }
}
