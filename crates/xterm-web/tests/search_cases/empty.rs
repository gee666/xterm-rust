// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use xterm_addons::search::{SearchDecorationOptionsHandle, SearchOptionsSource};

struct Poisoned {
    reads: Rc<Cell<usize>>,
    decorated: bool,
}
impl SearchOptionsSource for Poisoned {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        panic!("empty search must not snapshot matching flags")
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        self.reads.set(self.reads.get() + 1);
        Ok(self
            .decorated
            .then(|| options().read().unwrap().decorations.unwrap()))
    }
}

#[test]
fn empty_search_and_subsequent_writes_do_not_snapshot_matching_flags() {
    for reverse in [false, true] {
        for decorated in [false, true] {
            let h = Harness::new(20, 3, None);
            h.write("a a");
            h.addon.find_next(&term("a"), None).unwrap();
            let reads = Rc::new(Cell::new(0));
            let options: SearchOptionsHandle = Rc::new(Poisoned {
                reads: reads.clone(),
                decorated,
            });
            let found = if reverse {
                h.addon.find_previous(&[], Some(options))
            } else {
                h.addon.find_next(&[], Some(options))
            };
            assert!(!found.unwrap());
            assert!(h.selection().is_none());
            assert_eq!(reads.get(), 2);
            h.write("b");
            assert_eq!(reads.get(), 2);
        }
    }
}

struct Reentrant {
    addon: xterm_addons::search::WeakSearchAddon,
    entered: Cell<bool>,
}
impl SearchOptionsSource for Reentrant {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        panic!("empty search must not snapshot matching flags")
    }
    fn decorations(&self) -> Result<Option<SearchDecorationOptionsHandle>, SearchAddonError> {
        if !self.entered.replace(true) {
            assert!(self.addon.upgrade().unwrap().find_next(&term("a"), None)?);
        }
        Ok(None)
    }
}

#[test]
fn empty_decoration_getter_can_start_nested_search_without_state_borrow() {
    let h = Harness::new(20, 3, None);
    h.write("a");
    let options = Rc::new(Reentrant {
        addon: h.addon.downgrade(),
        entered: Cell::new(false),
    });
    assert!(!h.addon.find_next(&[], Some(options.clone())).unwrap());
    assert!(options.entered.get());
    assert!(h.selection().is_none());
}
