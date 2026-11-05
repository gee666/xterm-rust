// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource};

type Flags = dyn Fn() -> Result<xterm_core::search::SearchOptions, SearchAddonError>;
type Decorations = dyn Fn() -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError>;
struct PhasedOptions {
    flags: Rc<Flags>,
    decorations: Rc<Decorations>,
}
impl SearchOptionsSource for PhasedOptions {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        let decorations = self.decorations()?;
        Ok(SearchOptions {
            matching: self.matching_options()?,
            decorations,
        })
    }
    fn matching_options(&self) -> Result<xterm_core::search::SearchOptions, SearchAddonError> {
        (self.flags)()
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        (self.decorations)()
    }
}

#[test]
fn matching_failure_skips_selection_decoration_and_internal_scroll_reads() {
    for reverse in [false, true] {
        let h = Harness::new(20, 3, None);
        h.write("foo FOO");
        h.addon.find_next(&term("foo"), None).unwrap();
        let reads = Rc::new(Cell::new(0));
        let count = Rc::clone(&reads);
        let request = Rc::new(PhasedOptions {
            decorations: Rc::new(move || {
                count.set(count.get() + 1);
                assert_eq!(count.get(), 1, "only initial eligibility before failure");
                Ok(None)
            }),
            flags: Rc::new(|| Err(SearchAddonError::InvalidCoordinates)),
        });
        let result = h.addon.find_term_with_scroll(
            Rc::new(Text(term("foo"))),
            Some(request),
            reverse,
            || panic!("matching failure must skip noScroll"),
        );
        assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
        assert_eq!(reads.get(), 1);
        assert!(h.selection().is_none());
    }
}

struct Text(Vec<u16>);
impl xterm_addons::search::SearchTermSource for Text {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        Ok(!self.0.is_empty())
    }
    fn is_truthy(&self) -> bool {
        !self.0.is_empty()
    }
    fn equals(&self, other: &dyn xterm_addons::search::SearchTermSource) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.0 == other.0)
    }
    fn text(&self) -> Result<std::borrow::Cow<'_, [u16]>, SearchAddonError> {
        Ok(std::borrow::Cow::Borrowed(&self.0))
    }
}

#[test]
fn selection_decoration_getter_can_reenter_after_matching_before_no_scroll() {
    for reverse in [false, true] {
        for query in ["foo", "absent"] {
            for fail in [false, true] {
                let h = Harness::new(20, 3, None);
                h.write("foo FOO");
                h.addon.find_next(&term("foo"), None).unwrap();
                let reads = Rc::new(Cell::new(0));
                let flags_read = Rc::new(Cell::new(false));
                let count = Rc::clone(&reads);
                let flags = Rc::clone(&flags_read);
                let addon = h.addon.downgrade();
                let host = Rc::clone(&h.host);
                let active = options().decorations().unwrap();
                let request = Rc::new(PhasedOptions {
                    flags: Rc::new(move || {
                        flags.set(true);
                        Ok(xterm_core::search::SearchOptions {
                            case_sensitive: Some(true),
                            ..Default::default()
                        })
                    }),
                    decorations: Rc::new(move || {
                        count.set(count.get() + 1);
                        if count.get() != 2 {
                            return Ok(None);
                        }
                        assert!(flags_read.get(), "matching precedes selection decorations");
                        assert!(host.terminal.selection_position()?.is_none());
                        let addon = addon.upgrade().unwrap();
                        let mut nested = SearchOptions::default();
                        nested.matching.case_sensitive = Some(true);
                        assert!(addon.find_next(&term("FOO"), Some(Rc::new(RefCell::new(nested))))?);
                        // Mutating real cells proves no engine/cache borrow crosses the getter.
                        host.terminal.native.write("\rfoo FOO")?;
                        if fail {
                            return Err(SearchAddonError::InvalidCoordinates);
                        }
                        Ok(active.clone())
                    }),
                });
                let no_scroll = Cell::new(false);
                let result = h.addon.find_term_with_scroll(
                    Rc::new(Text(term(query))),
                    Some(request),
                    reverse,
                    || {
                        no_scroll.set(true);
                        assert_eq!(reads.get(), 2);
                        assert_eq!(h.selected_text(), term("FOO"));
                        Ok(true)
                    },
                );
                if fail {
                    assert!(matches!(result, Err(SearchAddonError::InvalidCoordinates)));
                    assert!(!no_scroll.get());
                    assert_eq!(h.selected_text(), term("FOO"));
                    assert_eq!(reads.get(), 2);
                } else {
                    assert_eq!(result.unwrap(), query == "foo");
                    assert!(no_scroll.get());
                    assert_eq!(reads.get(), 3, "initial, selection, result event");
                    assert_eq!(
                        h.selected_text(),
                        if query == "foo" { term("foo") } else { vec![] }
                    );
                    assert_eq!(h.decorations().len(), usize::from(query == "foo"));
                }
            }
        }
    }
}
