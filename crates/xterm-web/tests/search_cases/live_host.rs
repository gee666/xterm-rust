// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource, SearchTermSource};

fn select(h: &Harness, col: isize) {
    h.host
        .terminal
        .select(&SearchResult {
            term: term("foo"),
            row: 0,
            col,
            size: 3,
        })
        .unwrap();
}
fn pair() -> (Harness, Harness) {
    let old = Harness::new(20, 3, None);
    let current = Harness::new(20, 3, None);
    old.write("foo foo foo");
    current.write("foo foo foo");
    assert!(old.addon.find_next(&term("foo"), None).unwrap());
    select(&old, 0);
    select(&current, 4);
    (old, current)
}
fn find(h: &Harness, reverse: bool, query: &str, request: Option<SearchOptionsHandle>) -> bool {
    if reverse {
        h.addon.find_previous(&term(query), request)
    } else {
        h.addon.find_next(&term(query), request)
    }
    .unwrap()
}
fn assert_current(old: &Harness, current: &Harness, query: &str, col: isize) {
    assert_eq!(
        old.selection().unwrap().start.col,
        0,
        "entry host untouched"
    );
    assert_eq!(current.selection().is_some(), query == "foo");
    if query == "foo" {
        assert_eq!(current.selection().unwrap().start.col, col);
    }
}

#[test]
fn before_search_reactivation_uses_current_selection_and_clear_target() {
    for reverse in [false, true] {
        for query in ["foo", "absent", ""] {
            let (old, current) = pair();
            let weak = old.addon.downgrade();
            let host = Rc::clone(&current.host);
            old.addon.on_before_search().subscribe(move |_| {
                weak.upgrade().unwrap().activate(host.clone()).unwrap();
            });
            assert_eq!(find(&old, reverse, query, None), query == "foo");
            assert_current(&old, &current, query, if reverse { 0 } else { 8 });
        }
    }
}

type Read = dyn Fn() -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError>;
struct ReactiveOptions(Rc<Read>);
impl SearchOptionsSource for ReactiveOptions {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        Ok(SearchOptions {
            decorations: self.decorations()?,
            ..Default::default()
        })
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        (self.0)()
    }
    fn matching_options(&self) -> Result<xterm_core::search::SearchOptions, SearchAddonError> {
        Ok(Default::default())
    }
}
#[test]
fn decoration_reactivation_resolves_host_only_before_engine_selection_capture() {
    for reverse in [false, true] {
        for phase in [1, 2, 3, 4] {
            for query in ["foo", "absent", ""] {
                if phase != 1 && query.is_empty() {
                    continue;
                }
                let (old, current) = pair();
                let highlight = phase == 2 || phase == 3;
                if highlight {
                    old.addon.clear_decorations(false).unwrap();
                }
                let count = Rc::new(Cell::new(0));
                let reads = Rc::clone(&count);
                let weak = old.addon.downgrade();
                let host = Rc::clone(&current.host);
                let decorations = options().decorations().unwrap();
                let request = Rc::new(ReactiveOptions(Rc::new(move || {
                    reads.set(reads.get() + 1);
                    let at = if phase == 4 { 2 } else { phase };
                    if reads.get() == at {
                        weak.upgrade().unwrap().activate(host.clone())?;
                    }
                    Ok(if highlight { decorations.clone() } else { None })
                })));
                assert_eq!(find(&old, reverse, query, Some(request)), query == "foo");
                if phase == 4 {
                    assert!(
                        old.selection().is_none(),
                        "late getter follows old engine clear"
                    );
                    assert_eq!(current.selection().is_some(), query == "foo");
                    if query == "foo" {
                        assert_eq!(
                            current.selection().unwrap().start.col,
                            if reverse { 8 } else { 4 }
                        );
                    }
                } else {
                    assert_current(
                        &old,
                        &current,
                        query,
                        if highlight {
                            4
                        } else if reverse {
                            0
                        } else {
                            8
                        },
                    );
                }
                assert!(count.get() >= if highlight { 5 } else { 2 });
            }
        }
    }
}

struct InvalidTerm {
    read: Rc<dyn Fn()>,
}
impl SearchTermSource for InvalidTerm {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        (self.read)();
        Ok(false)
    }
    fn is_truthy(&self) -> bool {
        true
    }
    fn equals(&self, _: &dyn SearchTermSource) -> bool {
        false
    }
    fn text(&self) -> Result<std::borrow::Cow<'_, [u16]>, SearchAddonError> {
        panic!("invalid term must not reach matching")
    }
}
#[test]
fn invalid_term_validation_reactivation_clears_current_host() {
    for reverse in [false, true] {
        for at in [1, 2] {
            let (old, current) = pair();
            let count = Rc::new(Cell::new(0));
            let reads = Rc::clone(&count);
            let weak = old.addon.downgrade();
            let host = Rc::clone(&current.host);
            let raw = Rc::new(InvalidTerm {
                read: Rc::new(move || {
                    reads.set(reads.get() + 1);
                    if reads.get() == at {
                        weak.upgrade().unwrap().activate(host.clone()).unwrap();
                    }
                }),
            });
            assert!(!old
                .addon
                .find_term_with_scroll(raw, Some(options()), reverse, || Ok(false))
                .unwrap());
            assert_eq!(count.get(), 2);
            assert_current(&old, &current, "", 0);
        }
    }
}
