// Copyright (c) 2017, 2026 The xterm.js authors. All rights reserved. MIT license.
use super::*;
use std::{any::Any, borrow::Cow, error::Error};
use xterm_addons::search::{SearchHost, SearchOptionsSource, SearchTermSource, TimeoutCallback};
use xterm_core::{event::EmitError, lifecycle::DisposableHandle};

type Trace = Rc<RefCell<Vec<&'static str>>>;
struct TimerHost {
    base: Rc<search_support::Host>,
    failure: RefCell<Option<Rc<dyn Error>>>,
    trace: Trace,
    clock_reads: Cell<usize>,
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
        self.clock_reads.set(self.clock_reads.get() + 1);
        self.base.clock.now.set(self.base.clock.now.get() + 1);
        self.base.now_ms()
    }
    fn timeout(
        &self,
        delay: u64,
        callback: TimeoutCallback,
    ) -> Result<DisposableHandle, SearchAddonError> {
        self.trace.borrow_mut().push("timeout");
        assert_eq!(delay, xterm_core::search::CACHE_TTL_MS);
        if let Some(error) = self.failure.borrow().clone() {
            return Err(SearchAddonError::Host(error));
        }
        self.base.timeout(delay, callback)
    }
}
struct Guard {
    hook: Box<dyn Fn() -> Result<bool, SearchAddonError>>,
}
impl SearchTermSource for Guard {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn is_valid(&self) -> Result<bool, SearchAddonError> {
        Ok(true)
    }
    fn is_valid_for_engine(&self) -> Result<bool, SearchAddonError> {
        (self.hook)()
    }
    fn is_truthy(&self) -> bool {
        true
    }
    fn equals(&self, other: &dyn SearchTermSource) -> bool {
        other
            .as_any()
            .downcast_ref::<Self>()
            .is_some_and(|other| std::ptr::eq(self, other))
    }
    fn text(&self) -> Result<Cow<'_, [u16]>, SearchAddonError> {
        panic!("matching failure must precede raw term conversion")
    }
}
struct MatchingFailure {
    trace: Trace,
    error: Rc<dyn Error>,
}
impl SearchOptionsSource for MatchingFailure {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn read(&self) -> Result<SearchOptions, SearchAddonError> {
        Ok(Default::default())
    }
    fn matching_options(&self) -> Result<xterm_core::search::SearchOptions, SearchAddonError> {
        self.trace.borrow_mut().push("matching");
        Err(SearchAddonError::Host(self.error.clone()))
    }
}

#[test]
fn retired_cache_setup_preserves_timer_errors_and_successful_setup_matching_order() {
    for reverse in [false, true] {
        for warm in [false, true] {
            for dispose in [false, true] {
                for mode in ["throw", "return", "empty"] {
                    let old = Harness::new(20, 3, None);
                    let other = Harness::new(20, 3, None);
                    old.write("a a a");
                    other.write("    a a");
                    let trace: Trace = Rc::default();
                    let host = Rc::new(TimerHost {
                        base: old.host.clone(),
                        failure: RefCell::default(),
                        trace: trace.clone(),
                        clock_reads: Cell::new(0),
                    });
                    old.addon.activate(host.clone()).unwrap();
                    if warm {
                        assert!(old.addon.find_next(&term("a"), None).unwrap());
                    }
                    trace.borrow_mut().clear();
                    let timer_error: Rc<dyn Error> = Rc::new(std::io::Error::other("timer"));
                    let matching_error: Rc<dyn Error> = Rc::new(std::io::Error::other("matching"));
                    if mode == "throw" {
                        *host.failure.borrow_mut() = Some(timer_error.clone());
                    }
                    let guard = Rc::new(Guard {
                        hook: Box::new({
                            let weak = old.addon.downgrade();
                            let host = other.host.clone();
                            let trace = trace.clone();
                            move || {
                                trace.borrow_mut().push("guard");
                                let addon = weak.upgrade().unwrap();
                                if dispose {
                                    addon.dispose()?;
                                } else {
                                    addon.activate(host.clone())?;
                                }
                                Ok(mode != "empty")
                            }
                        }),
                    });
                    let scrolled = Cell::new(false);
                    let result = old.addon.find_term_with_scroll(
                        guard,
                        Some(Rc::new(MatchingFailure {
                            trace: trace.clone(),
                            error: matching_error.clone(),
                        })),
                        reverse,
                        || {
                            scrolled.set(true);
                            Ok(false)
                        },
                    );
                    if mode == "empty" {
                        assert!(!result.unwrap());
                        assert!(scrolled.get());
                        assert_eq!(*trace.borrow(), ["guard"]);
                    } else {
                        let SearchAddonError::Host(actual) = result.unwrap_err() else {
                            panic!("error representation changed")
                        };
                        assert!(Rc::ptr_eq(
                            &actual,
                            if mode == "throw" {
                                &timer_error
                            } else {
                                &matching_error
                            }
                        ));
                        assert!(!scrolled.get());
                        let expected = if mode == "throw" {
                            vec!["guard", "timeout"]
                        } else {
                            vec!["guard", "timeout", "matching"]
                        };
                        assert_eq!(*trace.borrow(), expected);
                    }
                    assert_eq!(
                        host.clock_reads.get(),
                        usize::from(warm) + usize::from(mode != "empty")
                    );
                    assert_eq!(
                        old.host.clock.pending(),
                        0,
                        "RAII, not orphan liveness parity"
                    );
                    assert_eq!(other.host.clock.pending(), 0);
                    old.addon.dispose().unwrap();
                }
            }
        }
    }
}
