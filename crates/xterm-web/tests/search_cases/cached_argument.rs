// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use std::{any::Any, borrow::Cow};
use xterm_addons::search::{SearchOptionsSource, SearchTermHandle, SearchTermSource};

type Hook = dyn Fn() -> Result<(), SearchAddonError>;
struct RawTerm {
    id: u8,
    text: Vec<u16>,
    guard: Rc<Hook>,
    matching: Rc<Hook>,
}
impl SearchTermSource for RawTerm {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        Ok(true)
    }
    fn is_valid_for_engine(&self) -> Result<bool, SearchAddonError> {
        (self.guard)()?;
        Ok(true)
    }
    fn is_truthy(&self) -> bool {
        true
    }
    fn equals(&self, other: &dyn SearchTermSource) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.id == other.id)
    }
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError> {
        (self.matching)()?;
        Ok(Cow::Borrowed(&self.text))
    }
}
struct Request(Rc<Hook>);
impl SearchOptionsSource for Request {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        Ok(Default::default())
    }
    fn matching_options(&self) -> Result<xterm_core::search::SearchOptions, SearchAddonError> {
        (self.0)()?;
        Ok(Default::default())
    }
}
fn plain(id: u8) -> SearchTermHandle {
    Rc::new(RawTerm {
        id,
        text: term(if id == 1 { "a" } else { "b" }),
        guard: Rc::new(|| Ok(())),
        matching: Rc::new(|| Ok(())),
    })
}
fn search(
    h: &Harness,
    raw: SearchTermHandle,
    reverse: bool,
    request: Option<SearchOptionsHandle>,
) -> Result<bool, SearchAddonError> {
    h.addon
        .find_term_with_scroll(raw, request, reverse, || Ok(true))
}
fn seed(h: &Harness, cached: Option<u8>, reverse: bool) {
    h.write("a a a");
    if let Some(id) = cached {
        search(h, plain(id), false, None).unwrap();
    }
    h.host
        .terminal
        .select(&SearchResult {
            term: term("a"),
            col: if reverse { 2 } else { 0 },
            row: 0,
            size: 1,
        })
        .unwrap();
}

#[test]
fn engine_cached_argument_precedes_guard_flags_and_text_reentry() {
    for phase in 0..3 {
        for reverse in [false, true] {
            for cached in [None, Some(1), Some(2)] {
                for nested in [None, Some(1), Some(2)] {
                    let h = Harness::new(20, 3, None);
                    seed(&h, cached, reverse);
                    let calls = Rc::new(Cell::new(0));
                    let hook: Rc<Hook> = Rc::new({
                        let weak = h.addon.downgrade();
                        let calls = calls.clone();
                        let host = h.host.clone();
                        move || {
                            calls.set(calls.get() + 1);
                            let addon = weak.upgrade().unwrap();
                            if let Some(id) = nested {
                                addon.find_term_with_scroll(plain(id), None, false, || Ok(true))?;
                            } else {
                                addon.clear_decorations(false)?;
                            }
                            if phase == 0 {
                                host.terminal.select(&SearchResult {
                                    term: term("a"),
                                    col: if reverse { 2 } else { 0 },
                                    row: 0,
                                    size: 1,
                                })?;
                            }
                            Ok(())
                        }
                    });
                    let noop: Rc<Hook> = Rc::new(|| Ok(()));
                    let raw = Rc::new(RawTerm {
                        id: 1,
                        text: term("a"),
                        guard: if phase == 0 {
                            hook.clone()
                        } else {
                            noop.clone()
                        },
                        matching: if phase == 2 {
                            hook.clone()
                        } else {
                            noop.clone()
                        },
                    });
                    let request = Rc::new(Request(if phase == 1 { hook } else { noop }));
                    assert!(search(&h, raw, reverse, Some(request)).unwrap());
                    let expected = match (reverse, cached == Some(1)) {
                        (false, true) | (true, false) => 2,
                        _ => 0,
                    };
                    assert_eq!(
                        h.selection().unwrap().start.col,
                        expected,
                        "phase={phase} reverse={reverse} cached={cached:?} nested={nested:?}"
                    );
                    assert_eq!(calls.get(), 1);
                    // Successful outer completion still replaces the nested cache.
                    assert!(search(&h, plain(1), false, None).unwrap());
                    assert_eq!(h.selection().unwrap().start.col, expected + 2);
                }
            }
        }
    }
}

#[test]
fn thrown_engine_callback_keeps_nested_cached_term_and_selection() {
    for phase in 0..3 {
        for reverse in [false, true] {
            let h = Harness::new(20, 3, None);
            seed(&h, Some(2), reverse);
            let hook: Rc<Hook> = Rc::new({
                let weak = h.addon.downgrade();
                move || {
                    let addon = weak.upgrade().unwrap();
                    addon.find_term_with_scroll(plain(1), None, false, || Ok(true))?;
                    Err(SearchAddonError::InvalidCoordinates)
                }
            });
            let noop: Rc<Hook> = Rc::new(|| Ok(()));
            let raw = Rc::new(RawTerm {
                id: 1,
                text: term("a"),
                guard: if phase == 0 {
                    hook.clone()
                } else {
                    noop.clone()
                },
                matching: if phase == 2 {
                    hook.clone()
                } else {
                    noop.clone()
                },
            });
            let request = Rc::new(Request(if phase == 1 { hook } else { noop }));
            assert!(matches!(
                search(&h, raw, reverse, Some(request)),
                Err(SearchAddonError::InvalidCoordinates)
            ));
            // Guard reentry happens before outer selection clear. Later phases
            // happen after clear, so their nested search begins at the top.
            let expected = if phase == 0 {
                if reverse {
                    2
                } else {
                    0
                }
            } else {
                0
            };
            assert_eq!(h.selection().unwrap().start.col, expected);
            assert!(search(&h, plain(1), false, None).unwrap());
            assert_eq!(h.selection().unwrap().start.col, expected + 2);
        }
    }
}
