// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use std::any::Any;
use xterm_addons::search::{SearchHost, SearchOptionsSource, TimeoutCallback};
use xterm_core::{
    event::EmitError,
    lifecycle::{to_disposable, DisposableHandle},
};

type Hook = dyn Fn() -> Result<(), SearchAddonError>;
struct TimerHost {
    base: Rc<search_support::Host>,
    installing: RefCell<Option<Rc<Hook>>>,
    cancelling: Rc<RefCell<Option<Rc<Hook>>>>,
}
impl SearchHost for TimerHost {
    fn terminal(&self) -> Rc<xterm_core::headless::Terminal> {
        self.base.terminal()
    }
    fn decorations(&self) -> Rc<xterm_core::decoration::DecorationService> {
        self.base.decorations()
    }
    fn selection(&self) -> Result<Option<SearchSelection>, SearchAddonError> {
        self.base.selection()
    }
    fn clear_selection(&self) -> Result<(), SearchAddonError> {
        self.base.clear_selection()
    }
    fn select(&self, result: &SearchResult) -> Result<(), SearchAddonError> {
        self.base.select(result)
    }
    fn style_decoration(
        &self,
        element: &Rc<dyn Any>,
        border: Option<&str>,
    ) -> Result<(), EmitError> {
        self.base.style_decoration(element, border)
    }
    fn now_ms(&self) -> u64 {
        self.base.now_ms()
    }
    fn timeout(
        &self,
        delay: u64,
        callback: TimeoutCallback,
    ) -> Result<DisposableHandle, SearchAddonError> {
        let timer = self.base.timeout(delay, callback)?;
        let hook = self.installing.borrow_mut().take();
        if let Some(hook) = hook {
            hook()?;
        }
        let cancelling = self.cancelling.clone();
        Ok(to_disposable(move || {
            timer.dispose();
            let hook = cancelling.borrow_mut().take();
            if let Some(hook) = hook {
                hook().unwrap();
            }
        }))
    }
}
struct InspectCache(Rc<Hook>);
impl SearchOptionsSource for InspectCache {
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

#[test]
fn activation_during_timer_install_cancels_retired_timer_before_matching() {
    let old = Harness::new(20, 3, None);
    let current = Harness::new(20, 3, None);
    old.write("a a a");
    current.write("    a a");
    let host = Rc::new(TimerHost {
        base: old.host.clone(),
        installing: RefCell::new(None),
        cancelling: Rc::new(RefCell::new(None)),
    });
    old.addon.activate(host.clone()).unwrap();
    *host.installing.borrow_mut() = Some(Rc::new({
        let weak = old.addon.downgrade();
        let current = current.host.clone();
        move || weak.upgrade().unwrap().activate(current.clone())
    }));
    let inspected = Rc::new(Cell::new(false));
    let request = Rc::new(InspectCache(Rc::new({
        let old = old.host.clone();
        let current = current.host.clone();
        let inspected = inspected.clone();
        move || {
            assert_eq!(
                old.clock.pending(),
                0,
                "disposed cache immediately cancels new timer"
            );
            assert_eq!(current.clock.pending(), 0);
            inspected.set(true);
            Ok(())
        }
    })));
    assert!(old.addon.find_next(&term("a"), Some(request)).unwrap());
    assert!(inspected.get());
    assert_eq!(current.selection().unwrap().start.col, 0);
    assert_eq!(old.host.clock.pending(), 0);
    assert!(old.addon.find_next(&term("a"), None).unwrap());
    assert_eq!(current.selection().unwrap().start.col, 4);
    assert_eq!(current.host.clock.pending(), 1);
    old.addon.dispose().unwrap();
    assert_eq!(current.host.clock.pending(), 0);
}

#[test]
fn cache_timer_cancellation_can_reactivate_without_an_addon_state_borrow() {
    let old = Harness::new(20, 3, None);
    let current = Harness::new(20, 3, None);
    old.write("a a a");
    current.write("    a a");
    let host = Rc::new(TimerHost {
        base: old.host.clone(),
        installing: RefCell::new(None),
        cancelling: Rc::new(RefCell::new(None)),
    });
    old.addon.activate(host.clone()).unwrap();
    assert!(old.addon.find_next(&term("a"), None).unwrap());
    assert_eq!(old.host.clock.pending(), 1);
    *host.cancelling.borrow_mut() = Some(Rc::new({
        let weak = old.addon.downgrade();
        let host = current.host.clone();
        move || weak.upgrade().unwrap().activate(host.clone())
    }));
    old.addon.dispose().unwrap();
    assert_eq!(old.host.clock.pending(), 0);
    assert!(host.cancelling.borrow().is_none(), "disposal callback ran");
    assert_eq!(current.host.clock.pending(), 0);
}

#[test]
fn reentrant_activation_disposal_keeps_addon_host_distinct_from_outer_engine() {
    let old = Harness::new(20, 3, None);
    let outer = Harness::new(20, 3, None);
    let nested = Harness::new(20, 3, None);
    old.write("a a a");
    outer.write("    a a");
    nested.write("        a a");
    let host = Rc::new(TimerHost {
        base: old.host.clone(),
        installing: RefCell::new(None),
        cancelling: Rc::new(RefCell::new(None)),
    });
    old.addon.activate(host.clone()).unwrap();
    assert!(old.addon.find_next(&term("a"), None).unwrap());
    let nested_host = Rc::new(TimerHost {
        base: nested.host.clone(),
        installing: RefCell::new(None),
        cancelling: Rc::new(RefCell::new(None)),
    });
    let cancelled = Rc::new(Cell::new(false));
    *nested_host.cancelling.borrow_mut() = Some(Rc::new({
        let weak = old.addon.downgrade();
        let cancelled = cancelled.clone();
        move || {
            cancelled.set(true);
            assert!(weak.upgrade().unwrap().find_next(&term("a"), None)?);
            Ok(())
        }
    }));
    *host.cancelling.borrow_mut() = Some(Rc::new({
        let weak = old.addon.downgrade();
        let host = nested_host.clone();
        move || {
            let addon = weak.upgrade().unwrap();
            addon.activate(host.clone())?;
            assert!(addon.find_next(&term("a"), None)?);
            Ok(())
        }
    }));
    old.addon.activate(outer.host.clone()).unwrap();
    assert!(
        cancelled.get(),
        "replaced nested engine cancelled its live timer"
    );
    assert!(old.addon.find_next(&term("a"), None).unwrap());
    assert!(
        outer.selection().is_none(),
        "engine selection is cleared, not selected"
    );
    assert_eq!(
        nested.selection().unwrap().start.col,
        4,
        "live addon selects the outer engine result"
    );
    assert_eq!(outer.host.clock.pending(), 1);
    assert_eq!(nested.host.clock.pending(), 0);
    assert_eq!(old.host.clock.pending(), 0);
    old.addon.dispose().unwrap();
    assert_eq!(outer.host.clock.pending(), 0);
}
