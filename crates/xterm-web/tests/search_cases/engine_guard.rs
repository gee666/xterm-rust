// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use std::{any::Any, borrow::Cow};
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource, SearchTermSource};

type Guard = dyn Fn() -> Result<bool, SearchAddonError>;
struct RawTerm {
    guard: Rc<Guard>,
    text: Rc<Guard>,
}
impl SearchTermSource for RawTerm {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        Ok(true)
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
    decorations: Rc<Cell<usize>>,
    flags: Rc<Cell<usize>>,
}
impl SearchOptionsSource for Request {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        Ok(Default::default())
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        self.decorations.set(self.decorations.get() + 1);
        Ok(None)
    }
    fn matching_options(&self) -> Result<xterm_core::search::SearchOptions, SearchAddonError> {
        self.flags.set(self.flags.get() + 1);
        Ok(Default::default())
    }
}
fn seed(h: &Harness) {
    h.write("foo foo foo");
    assert!(h.addon.find_next(&term("foo"), Some(options())).unwrap());
    h.host.clock.advance(30_000).unwrap();
    assert_eq!(h.host.clock.pending(), 0);
    assert_eq!(h.decorations().len(), 4);
}
fn forbidden_text() -> Rc<Guard> {
    Rc::new(|| panic!("empty or throwing engine guard must not read matching text"))
}

#[test]
fn empty_engine_guard_keeps_highlights_skips_matching_and_runs_selection_arguments() {
    for reverse in [false, true] {
        for reenter in [false, true] {
            let h = Harness::new(20, 3, None);
            let current = Harness::new(20, 3, None);
            seed(&h);
            seed(&current);
            let guard_calls = Rc::new(Cell::new(0));
            let calls = Rc::clone(&guard_calls);
            let host = Rc::clone(&current.host);
            let old_host = Rc::clone(&h.host);
            let weak = h.addon.downgrade();
            let raw = Rc::new(RawTerm {
                guard: Rc::new(move || {
                    calls.set(calls.get() + 1);
                    assert!(old_host.terminal.selection_position()?.is_some());
                    assert_eq!(old_host.clock.pending(), 0);
                    if reenter {
                        weak.upgrade().unwrap().activate(host.clone())?;
                    }
                    Ok(false)
                }),
                text: forbidden_text(),
            });
            let decorations = Rc::new(Cell::new(0));
            let flags = Rc::new(Cell::new(0));
            let request = Rc::new(Request {
                decorations: Rc::clone(&decorations),
                flags: Rc::clone(&flags),
            });
            let scroll = Cell::new(0);
            let after = Rc::new(Cell::new(0));
            let capture = Rc::clone(&after);
            h.addon
                .on_after_search()
                .subscribe(move |_| capture.set(capture.get() + 1));
            assert!(!h
                .addon
                .find_term_with_scroll(raw, Some(request), reverse, || {
                    scroll.set(scroll.get() + 1);
                    Ok(true)
                })
                .unwrap());
            assert_eq!(guard_calls.get(), 1);
            assert_eq!(flags.get(), 0);
            assert_eq!(decorations.get(), 3);
            assert_eq!(scroll.get(), 1);
            assert_eq!(after.get(), 1);
            assert!(h.selection().is_none(), "captured engine host is cleared");
            assert_eq!(current.selection().is_none(), reenter);
            assert_eq!(
                h.decorations().len(),
                3,
                "engine empty does not clear highlights"
            );
            assert_eq!(h.host.clock.pending(), 0);
        }
    }
}

#[test]
fn engine_guard_error_precedes_selection_clear_and_all_later_arguments() {
    for reverse in [false, true] {
        for reenter in [false, true] {
            let h = Harness::new(20, 3, None);
            let current = Harness::new(20, 3, None);
            seed(&h);
            seed(&current);
            let host = Rc::clone(&current.host);
            let weak = h.addon.downgrade();
            let raw = Rc::new(RawTerm {
                guard: Rc::new(move || {
                    if reenter {
                        weak.upgrade().unwrap().activate(host.clone())?;
                    }
                    Err(SearchAddonError::InvalidCoordinates)
                }),
                text: forbidden_text(),
            });
            let decorations = Rc::new(Cell::new(0));
            let flags = Rc::new(Cell::new(0));
            let request = Rc::new(Request {
                decorations: Rc::clone(&decorations),
                flags: Rc::clone(&flags),
            });
            let after = Rc::new(Cell::new(0));
            let capture = Rc::clone(&after);
            h.addon
                .on_after_search()
                .subscribe(move |_| capture.set(capture.get() + 1));
            let result = h
                .addon
                .find_term_with_scroll(raw, Some(request), reverse, || {
                    panic!("guard error must skip internal scroll argument")
                });
            assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
            assert!(h.selection().is_some());
            assert!(current.selection().is_some());
            assert_eq!(flags.get(), 0);
            assert_eq!(decorations.get(), 1);
            assert_eq!(after.get(), 0);
            assert_eq!(h.host.clock.pending(), 0);
        }
    }
}

#[test]
fn nonempty_guard_reads_text_only_after_selection_clear_cache_and_options() {
    for reverse in [false, true] {
        let h = Harness::new(20, 3, None);
        seed(&h);
        let host = Rc::clone(&h.host);
        let flags = Rc::new(Cell::new(0));
        let capture = Rc::clone(&flags);
        let raw = Rc::new(RawTerm {
            guard: Rc::new(|| Ok(true)),
            text: Rc::new(move || {
                assert!(host.terminal.selection_position()?.is_none());
                assert_eq!(host.clock.pending(), 1);
                assert_eq!(capture.get(), 1);
                Err(SearchAddonError::InvalidCoordinates)
            }),
        });
        let decorations = Rc::new(Cell::new(0));
        let request = Rc::new(Request {
            decorations: Rc::clone(&decorations),
            flags,
        });
        assert!(matches!(
            h.addon
                .find_term_with_scroll(raw, Some(request), reverse, || {
                    panic!("text error must skip scroll argument")
                }),
            Err(SearchAddonError::InvalidCoordinates)
        ));
        assert!(h.selection().is_none());
        assert_eq!(decorations.get(), 1);
        assert_eq!(h.decorations().len(), 4);
    }
}
