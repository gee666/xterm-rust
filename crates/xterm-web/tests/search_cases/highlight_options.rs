// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource};

type Read = dyn Fn() -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError>;
struct PhasedOptions {
    decorations: Rc<Read>,
    fail_matching: bool,
}
impl SearchOptionsSource for PhasedOptions {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        Ok(SearchOptions {
            decorations: self.decorations()?,
            matching: self.matching_options()?,
        })
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        (self.decorations)()
    }
    fn matching_options(&self) -> Result<xterm_core::search::SearchOptions, SearchAddonError> {
        if self.fail_matching {
            Err(SearchAddonError::InvalidCoordinates)
        } else {
            Ok(Default::default())
        }
    }
}
struct ObservedLimit(Rc<Cell<bool>>);
impl SearchHighlightLimit for ObservedLimit {
    fn reached(&self, _: usize) -> Result<bool, SearchAddonError> {
        Ok(false)
    }
    fn end_index(&self, length: usize) -> Result<usize, SearchAddonError> {
        self.0.set(true);
        Ok(length)
    }
}
fn observed() -> (Harness, Rc<Cell<bool>>) {
    let mut h = Harness::new(20, 3, None);
    h.addon.dispose().unwrap();
    let ended = Rc::new(Cell::new(false));
    h.addon = SearchAddon::with_limit(Rc::new(ObservedLimit(Rc::clone(&ended))));
    h.addon.activate(h.host.clone()).unwrap();
    h.write("foo FOO");
    h.host
        .terminal
        .select(&SearchResult {
            term: term("foo"),
            col: 0,
            row: 0,
            size: 3,
        })
        .unwrap();
    (h, ended)
}
fn sensitive() -> SearchOptionsHandle {
    let mut options = SearchOptions::default();
    options.matching.case_sensitive = Some(true);
    Rc::new(RefCell::new(options))
}

#[test]
fn highlight_eligibility_and_argument_follow_results_and_remain_live() {
    for reverse in [false, true] {
        for query in ["foo", "absent"] {
            for skip in [false, true] {
                for nested in [false, true] {
                    let (h, ended) = observed();
                    let count = Rc::new(Cell::new(0));
                    let reads = Rc::clone(&count);
                    let weak = h.addon.downgrade();
                    let host = Rc::clone(&h.host);
                    let eligibility = options().decorations().unwrap();
                    let argument = Rc::new(xterm_addons::search::SearchDecorationOptions {
                        match_background: Some("#aabbcc".into()),
                        match_overview_ruler: "#ff0000".into(),
                        ..Default::default()
                    }) as SearchDecorationOptionsHandle;
                    let request = Rc::new(PhasedOptions {
                        fail_matching: false,
                        decorations: Rc::new(move || {
                            let read = reads.get() + 1;
                            reads.set(read);
                            if read == 1 {
                                return Ok(eligibility.clone());
                            }
                            if read == 2 {
                                assert!(ended.get(), "results must be sliced first");
                                assert!(host.terminal.selection_position()?.is_some());
                                if nested {
                                    let addon = weak.upgrade().unwrap();
                                    assert!(addon.find_next(&term("FOO"), Some(sensitive()))?);
                                    // Reentry and cell mutation must not meet a live engine borrow.
                                    host.terminal.native.write("\rfoo FOO")?;
                                }
                                return Ok(if skip { None } else { eligibility.clone() });
                            }
                            if !skip && read == 3 {
                                return Ok(Some(Rc::clone(&argument)));
                            }
                            Ok(None)
                        }),
                    });
                    let found = if reverse {
                        h.addon.find_previous(&term(query), Some(request))
                    } else {
                        h.addon.find_next(&term(query), Some(request))
                    }
                    .unwrap();
                    assert_eq!(found, query == "foo");
                    assert_eq!(count.get(), if skip { 4 } else { 5 });
                    let bottom = h.decorations();
                    assert_eq!(bottom.len(), if skip || query == "absent" { 0 } else { 2 });
                    for decoration in bottom {
                        assert_eq!(
                            decoration.options().borrow().background_color.as_deref(),
                            Some("#aabbcc")
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn highlight_getter_failures_cut_off_selection_and_preserve_reentry() {
    for reverse in [false, true] {
        for fail_at in [2, 3] {
            for nested in [false, true] {
                let (h, ended) = observed();
                let count = Rc::new(Cell::new(0));
                let reads = Rc::clone(&count);
                let weak = h.addon.downgrade();
                let eligibility = options().decorations().unwrap();
                let request = Rc::new(PhasedOptions {
                    fail_matching: false,
                    decorations: Rc::new(move || {
                        reads.set(reads.get() + 1);
                        if reads.get() == fail_at {
                            assert!(ended.get());
                            if nested {
                                assert!(weak
                                    .upgrade()
                                    .unwrap()
                                    .find_next(&term("FOO"), Some(sensitive()))?);
                            }
                            return Err(SearchAddonError::InvalidCoordinates);
                        }
                        Ok(eligibility.clone())
                    }),
                });
                let result = if reverse {
                    h.addon.find_previous(&term("foo"), Some(request))
                } else {
                    h.addon.find_next(&term("foo"), Some(request))
                };
                assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
                assert_eq!(count.get(), fail_at);
                assert_eq!(h.selection().unwrap().start.col, if nested { 4 } else { 0 });
                assert!(h.decorations().is_empty());
            }
        }
    }
}

#[test]
fn absent_second_highlight_argument_only_fails_when_a_result_reads_colors() {
    for reverse in [false, true] {
        for query in ["foo", "absent"] {
            let (h, _) = observed();
            let count = Rc::new(Cell::new(0));
            let reads = Rc::clone(&count);
            let eligibility = options().decorations().unwrap();
            let request = Rc::new(PhasedOptions {
                fail_matching: false,
                decorations: Rc::new(move || {
                    reads.set(reads.get() + 1);
                    Ok(if reads.get() < 3 {
                        eligibility.clone()
                    } else {
                        None
                    })
                }),
            });
            let result = if reverse {
                h.addon.find_previous(&term(query), Some(request))
            } else {
                h.addon.find_next(&term(query), Some(request))
            };
            if query == "foo" {
                assert!(matches!(result, Err(SearchAddonError::Dispatch(_))));
                assert_eq!(count.get(), 3);
                assert_eq!(h.selected_text(), term("foo"));
            } else {
                assert!(!result.unwrap());
                assert_eq!(count.get(), 5);
            }
        }
    }
}

#[test]
fn highlight_manager_is_captured_between_eligibility_and_argument_getters() {
    for reverse in [false, true] {
        for activate_at in [2, 3] {
            let (h, _) = observed();
            let other = Harness::new(20, 3, None);
            other.write("foo FOO");
            let other_host = Rc::clone(&other.host);
            let weak = h.addon.downgrade();
            let reads = Cell::new(0);
            let colors = options().decorations().unwrap();
            let request = Rc::new(PhasedOptions {
                fail_matching: false,
                decorations: Rc::new(move || {
                    reads.set(reads.get() + 1);
                    if reads.get() == activate_at {
                        weak.upgrade().unwrap().activate(other_host.clone())?;
                    }
                    Ok(if reads.get() <= 3 {
                        colors.clone()
                    } else {
                        None
                    })
                }),
            });
            let result = if reverse {
                h.addon.find_previous(&term("foo"), Some(request))
            } else {
                h.addon.find_next(&term("foo"), Some(request))
            };
            assert!(result.unwrap());
            assert_eq!(h.decorations().len(), if activate_at == 3 { 2 } else { 0 });
            assert_eq!(
                other.decorations().len(),
                if activate_at == 2 { 2 } else { 0 }
            );
        }
    }
}

#[test]
fn highlight_matching_errors_skip_both_late_decoration_reads() {
    for reverse in [false, true] {
        let (h, ended) = observed();
        let count = Rc::new(Cell::new(0));
        let reads = Rc::clone(&count);
        let eligibility = options().decorations().unwrap();
        let request = Rc::new(PhasedOptions {
            fail_matching: true,
            decorations: Rc::new(move || {
                reads.set(reads.get() + 1);
                Ok(eligibility.clone())
            }),
        });
        let result = if reverse {
            h.addon.find_previous(&term("foo"), Some(request))
        } else {
            h.addon.find_next(&term("foo"), Some(request))
        };
        assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
        assert_eq!(count.get(), 1);
        assert!(!ended.get());
        assert_eq!(h.selected_text(), term("foo"));
    }
}
