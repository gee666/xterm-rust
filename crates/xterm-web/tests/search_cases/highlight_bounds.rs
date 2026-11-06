// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use std::{any::Any, borrow::Cow};
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource, SearchTermSource};
use xterm_core::search::SearchError;

type Guard = dyn Fn() -> Result<bool, SearchAddonError>;
struct Term {
    guard: Rc<Guard>,
    texts: Rc<Cell<usize>>,
}
impl SearchTermSource for Term {
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
        self.texts.set(self.texts.get() + 1);
        Ok(Cow::Owned(term("foo")))
    }
}
struct Request {
    reads: Rc<Cell<usize>>,
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
        self.reads.set(self.reads.get() + 1);
        if self.reads.get() > 1 {
            return Err(SearchAddonError::MissingOptions);
        }
        Ok(Default::default())
    }
}

#[test]
fn highlight_bounds_follow_guard_but_precede_cache_options_and_text() {
    for reverse in [false, true] {
        for reactivate in [false, true] {
            for guard_throws in [false, true] {
                let h = Harness::new(20, 3, None);
                let other = Harness::new(20, 3, None);
                h.write("foo foo");
                let old = Rc::clone(&h.host);
                let new = Rc::clone(&other.host);
                let weak = h.addon.downgrade();
                let guards = Rc::new(Cell::new(0));
                let capture = Rc::clone(&guards);
                let query = Rc::new(Term {
                    guard: Rc::new(move || {
                        capture.set(capture.get() + 1);
                        if capture.get() == 2 {
                            assert_eq!(old.clock.pending(), 1);
                            if reactivate {
                                weak.upgrade().unwrap().activate(new.clone())?;
                            }
                            old.terminal.native.resize(2, 3)?;
                            assert_eq!(old.clock.pending(), 0);
                            if guard_throws {
                                return Err(SearchAddonError::InvalidCoordinates);
                            }
                        }
                        Ok(true)
                    }),
                    texts: Rc::new(Cell::new(0)),
                });
                let reads = Rc::new(Cell::new(0));
                let result = h.addon.find_term_with_scroll(
                    query.clone(),
                    Some(Rc::new(Request {
                        reads: Rc::clone(&reads),
                    })),
                    reverse,
                    || panic!("highlight failure must skip later selection options"),
                );
                if guard_throws {
                    assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
                } else {
                    assert!(
                        matches!(
                            result,
                            Err(SearchAddonError::Search(SearchError::InvalidColumn {
                                col: 3,
                                cols: 2
                            }))
                        ),
                        "unexpected error: {result:?}"
                    );
                }
                assert_eq!(guards.get(), 2);
                assert_eq!(reads.get(), 1);
                assert_eq!(query.texts.get(), 1);
                assert_eq!(h.host.clock.pending(), 0);
                assert_eq!(other.host.clock.pending(), 0);
                assert!(h.decorations().is_empty());
                assert!(other.decorations().is_empty());
            }
        }
    }
}
