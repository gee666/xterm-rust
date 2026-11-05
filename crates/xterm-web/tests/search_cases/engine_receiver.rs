// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use std::{any::Any, borrow::Cow};
use xterm_addons::search::{SearchOptionsSource, SearchTermSource};

type Hook = dyn Fn() -> Result<(), SearchAddonError>;
struct RawTerm {
    guard: Rc<Hook>,
    text: Rc<Hook>,
    value: Vec<u16>,
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
    fn equals(&self, _: &dyn SearchTermSource) -> bool {
        false
    }
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError> {
        (self.text)()?;
        Ok(Cow::Borrowed(&self.value))
    }
}
struct Request {
    matching: Rc<Hook>,
    highlight: bool,
}
impl SearchOptionsSource for Request {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        Ok(SearchOptions {
            decorations: if self.highlight {
                options().decorations()?
            } else {
                None
            },
            ..Default::default()
        })
    }
    fn matching_options(&self) -> Result<xterm_core::search::SearchOptions, SearchAddonError> {
        (self.matching)()?;
        Ok(Default::default())
    }
}
fn select(h: &Harness, col: isize) {
    h.host
        .terminal
        .select(&SearchResult {
            term: term("a"),
            row: 0,
            col,
            size: 1,
        })
        .unwrap();
}
fn pair(warm: bool) -> (Harness, Harness) {
    let old = Harness::new(20, 3, None);
    let current = Harness::new(20, 3, None);
    old.write("a a a");
    current.write("    a a");
    if warm {
        assert!(old.addon.find_next(&term("a"), None).unwrap());
    }
    select(&old, 0);
    select(&current, 4);
    (old, current)
}

#[test]
fn engine_receiver_keeps_old_host_and_cache_after_guard_flags_or_text_activation() {
    for phase in 0..3 {
        for reverse in [false, true] {
            for nested in [false, true] {
                for warm in [false, true] {
                    let (old, current) = pair(warm);
                    let armed = Rc::new(Cell::new(true));
                    let hook: Rc<Hook> = Rc::new({
                        let weak = old.addon.downgrade();
                        let host = current.host.clone();
                        let armed = armed.clone();
                        move || {
                            assert!(armed.replace(false));
                            let addon = weak.upgrade().unwrap();
                            addon.activate(host.clone())?;
                            if nested {
                                assert!(addon.find_next(&term("a"), None)?);
                            }
                            Ok(())
                        }
                    });
                    let noop: Rc<Hook> = Rc::new(|| Ok(()));
                    let raw = Rc::new(RawTerm {
                        guard: if phase == 0 {
                            hook.clone()
                        } else {
                            noop.clone()
                        },
                        text: if phase == 2 {
                            hook.clone()
                        } else {
                            noop.clone()
                        },
                        value: term("a"),
                    });
                    let request = Rc::new(Request {
                        matching: if phase == 1 { hook } else { noop },
                        highlight: false,
                    });
                    assert!(old
                        .addon
                        .find_term_with_scroll(raw, Some(request), reverse, || Ok(true))
                        .unwrap());
                    assert!(!armed.get());
                    assert!(old.selection().is_none());
                    // Raw identity differs from cached native text. The engine
                    // expands from the old start, then selects on the live host.
                    assert_eq!(current.selection().unwrap().start.col, 0);
                    assert_eq!(old.host.clock.pending(), 0);
                    assert_eq!(current.host.clock.pending(), usize::from(nested));
                    assert!(old.addon.find_next(&term("a"), None).unwrap());
                    assert_eq!(current.selection().unwrap().start.col, 4);
                    assert_eq!(current.host.clock.pending(), 1);
                    old.write("\x1b[H");
                    assert_eq!(
                        current.host.clock.pending(),
                        1,
                        "old events cannot invalidate new cache"
                    );
                    current.host.terminal.native.resize(21, 3).unwrap();
                    assert_eq!(
                        current.host.clock.pending(),
                        0,
                        "new events invalidate only new cache"
                    );
                    assert!(old.addon.find_next(&term("a"), None).unwrap());
                    assert_eq!(current.host.clock.pending(), 1);
                    old.host.clock.advance(20_000).unwrap();
                    assert_eq!(
                        current.host.clock.pending(),
                        1,
                        "old timers cannot expire new cache"
                    );
                    current.host.clock.advance(15_000).unwrap();
                    assert_eq!(current.host.clock.pending(), 0);
                }
            }
        }
    }
}

#[test]
fn throwing_engine_reactivation_keeps_live_host_without_reinstalling_old_cache_jobs() {
    for phase in 0..3 {
        for reverse in [false, true] {
            for nested in [false, true] {
                let (old, current) = pair(true);
                let hook: Rc<Hook> = Rc::new({
                    let weak = old.addon.downgrade();
                    let host = current.host.clone();
                    move || {
                        let addon = weak.upgrade().unwrap();
                        addon.activate(host.clone())?;
                        if nested {
                            assert!(addon.find_next(&term("a"), None)?);
                        }
                        Err(SearchAddonError::InvalidCoordinates)
                    }
                });
                let noop: Rc<Hook> = Rc::new(|| Ok(()));
                let raw = Rc::new(RawTerm {
                    guard: if phase == 0 {
                        hook.clone()
                    } else {
                        noop.clone()
                    },
                    text: if phase == 2 {
                        hook.clone()
                    } else {
                        noop.clone()
                    },
                    value: term("a"),
                });
                let request = Rc::new(Request {
                    matching: if phase == 1 { hook } else { noop },
                    highlight: false,
                });
                assert!(matches!(
                    old.addon
                        .find_term_with_scroll(raw, Some(request), reverse, || Ok(true)),
                    Err(SearchAddonError::InvalidCoordinates)
                ));
                assert_eq!(old.selection().is_some(), phase == 0);
                assert_eq!(
                    current.selection().unwrap().start.col,
                    if nested { 6 } else { 4 }
                );
                assert_eq!(old.host.clock.pending(), 0);
                assert_eq!(current.host.clock.pending(), usize::from(nested));
                assert!(old.addon.find_next(&term("a"), None).unwrap());
                assert_eq!(
                    current.selection().unwrap().start.col,
                    if nested { 4 } else { 6 }
                );
            }
        }
    }
}

#[test]
fn highlight_calls_capture_separate_engines_and_dispose_old_cache_timers() {
    for phase in 0..3 {
        for reverse in [false, true] {
            let (old, current) = pair(true);
            old.addon.clear_decorations(false).unwrap();
            let armed = Rc::new(Cell::new(true));
            let hook: Rc<Hook> = Rc::new({
                let weak = old.addon.downgrade();
                let host = current.host.clone();
                let armed = armed.clone();
                move || {
                    if armed.replace(false) {
                        weak.upgrade().unwrap().activate(host.clone())?;
                    }
                    Ok(())
                }
            });
            let noop: Rc<Hook> = Rc::new(|| Ok(()));
            let raw = Rc::new(RawTerm {
                guard: if phase == 0 {
                    hook.clone()
                } else {
                    noop.clone()
                },
                text: if phase == 2 {
                    hook.clone()
                } else {
                    noop.clone()
                },
                value: term("a"),
            });
            let request = Rc::new(Request {
                matching: if phase == 1 { hook } else { noop },
                highlight: true,
            });
            let changed = Rc::new(RefCell::new(Vec::new()));
            let capture = changed.clone();
            old.addon
                .on_did_change_results()
                .subscribe(move |event| capture.borrow_mut().push(*event));
            assert!(old
                .addon
                .find_term_with_scroll(raw, Some(request), reverse, || Ok(true))
                .unwrap());
            assert!(!armed.get());
            assert_eq!(changed.borrow().last().unwrap().result_count, 3);
            assert_eq!(old.host.clock.pending(), 0);
            assert_eq!(current.host.clock.pending(), 1);
            assert_eq!(current.selection().unwrap().start.col, 4);
            old.addon.dispose().unwrap();
            assert_eq!(current.host.clock.pending(), 0);
        }
    }
}
