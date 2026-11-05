// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use std::{any::Any, borrow::Cow};
use xterm_addons::search::{
    SearchDecorationOptionsHandle, SearchOptionsSource, SearchTermHandle, SearchTermSource,
    WeakSearchAddon,
};

struct InvalidTerm {
    identity: usize,
    truthy: bool,
    reads: Cell<usize>,
    nested: Option<WeakSearchAddon>,
}
impl SearchTermSource for InvalidTerm {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        let reads = self.reads.get();
        self.reads.set(reads + 1);
        if reads == 0 {
            if let Some(addon) = self.nested.as_ref().and_then(WeakSearchAddon::upgrade) {
                assert!(addon.find_next(&term("a"), None)?);
            }
        }
        Ok(false)
    }
    fn is_truthy(&self) -> bool {
        self.truthy
    }
    fn equals(&self, other: &dyn SearchTermSource) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| self.identity == other.identity)
    }
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError> {
        panic!("invalid term must not be coerced to matching text")
    }
}
struct Flags {
    comparisons: Cell<usize>,
    decorations: Cell<usize>,
}
impl SearchOptionsSource for Flags {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        panic!("invalid term must not snapshot matching flags")
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        self.decorations.set(self.decorations.get() + 1);
        Ok(options().read()?.decorations)
    }
    fn flags_changed(
        &self,
        _stored: &dyn Fn() -> Option<SearchOptionsHandle>,
    ) -> Result<bool, SearchAddonError> {
        self.comparisons.set(self.comparisons.get() + 1);
        Ok(false)
    }
}
fn invalid(identity: usize, truthy: bool, nested: Option<WeakSearchAddon>) -> Rc<InvalidTerm> {
    Rc::new(InvalidTerm {
        identity,
        truthy,
        reads: Cell::new(0),
        nested,
    })
}

#[test]
fn invalid_length_access_can_reenter_without_state_or_buffer_borrows() {
    for reverse in [false, true] {
        let h = Harness::new(20, 3, None);
        h.write("a a");
        let value = invalid(0, true, Some(h.addon.downgrade()));
        let input: SearchTermHandle = value.clone();
        assert!(!h
            .addon
            .find_term_with_scroll(input, Some(options()), reverse, || {
                panic!("invalid term must not read noScroll")
            })
            .unwrap());
        assert_eq!(value.reads.get(), 2);
        assert!(h.selection().is_none());
        assert!(h.decorations().is_empty());
    }
}

#[test]
fn invalid_cache_preserves_identity_and_raw_truthiness_on_write() {
    for truthy in [false, true] {
        let h = Harness::new(20, 3, None);
        h.write("a a");
        let request = Rc::new(Flags {
            comparisons: Cell::new(0),
            decorations: Cell::new(0),
        });
        for identity in [1, 1, 2] {
            assert!(!h
                .addon
                .find_term_with_scroll(
                    invalid(identity, truthy, None),
                    Some(request.clone()),
                    false,
                    || Ok(false)
                )
                .unwrap());
        }
        assert_eq!(request.comparisons.get(), 1);
        assert_eq!(request.decorations.get(), 6);
        h.write("b");
        assert_eq!(request.decorations.get(), 6 + usize::from(truthy));
    }
}
