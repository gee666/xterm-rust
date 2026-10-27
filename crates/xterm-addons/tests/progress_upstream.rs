// Copyright (c) 2024 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. The ten browser assertions use a fixture adapter
// here; this does not claim their Rust/WASM browser execution.
#[path = "support/status.rs"]
pub mod support;
use std::cell::RefCell;
use std::rc::Rc;
use support::FixtureTerminal;
use xterm_addons::progress::{ProgressAddon, ProgressState};
use xterm_core::lifecycle::Disposable;

type Events = Rc<RefCell<Vec<ProgressState>>>;
fn setup() -> (Rc<FixtureTerminal>, ProgressAddon, Events) {
    let terminal = Rc::new(FixtureTerminal::default());
    let addon = ProgressAddon::new();
    addon.activate(terminal.clone());
    let events = Rc::new(RefCell::new(Vec::new()));
    let capture = events.clone();
    addon
        .on_change()
        .unwrap()
        .subscribe(move |s| capture.borrow_mut().push(*s));
    (terminal, addon, events)
}
fn s(state: f64, value: f64) -> ProgressState {
    ProgressState { state, value }
}
#[test]
fn initial_values_are_zero_zero() {
    let (_, addon, _) = setup();
    assert_eq!(addon.progress(), s(0., 0.));
}
#[test]
fn state_zero_remove_ignores_value() {
    let (term, _addon, events) = setup();
    term.osc(9, "4;0");
    term.osc(9, "4;0;12");
    assert_eq!(*events.borrow(), [s(0., 0.), s(0., 0.)]);
}
#[test]
fn state_one_set() {
    let (term, _addon, events) = setup();
    for v in [10, 50, 23] {
        term.osc(9, &format!("4;1;{v}"));
    }
    assert_eq!(*events.borrow(), [s(1., 10.), s(1., 50.), s(1., 23.)]);
}
#[test]
fn state_one_special_sequences() {
    let (term, _addon, events) = setup();
    term.osc(9, "4;1");
    term.osc(9, "4;1;12x");
    term.osc(9, "4;1;123");
    assert_eq!(*events.borrow(), [s(1., 0.), s(1., 100.)]);
}
#[test]
fn error_preserves_previous_value_on_empty_or_zero() {
    let (term, _addon, events) = setup();
    for p in ["4;1;12", "4;2", "4;2;", "4;2;0"] {
        term.osc(9, p);
    }
    assert_eq!(
        *events.borrow(),
        [s(1., 12.), s(2., 12.), s(2., 12.), s(2., 12.)]
    );
}
#[test]
fn error_accepts_new_value() {
    let (term, _addon, events) = setup();
    for p in ["4;1;12", "4;2;25", "4;2;123"] {
        term.osc(9, p);
    }
    assert_eq!(*events.borrow(), [s(1., 12.), s(2., 25.), s(2., 100.)]);
}
#[test]
fn indeterminate_keeps_value() {
    let (term, _addon, events) = setup();
    for p in ["4;1;12", "4;3", "4;3;123"] {
        term.osc(9, p);
    }
    assert_eq!(*events.borrow(), [s(1., 12.), s(3., 12.), s(3., 12.)]);
}
#[test]
fn pause_preserves_previous_on_empty_or_zero() {
    let (term, _addon, events) = setup();
    for p in ["4;1;12", "4;4", "4;4;", "4;4;0"] {
        term.osc(9, p);
    }
    assert_eq!(
        *events.borrow(),
        [s(1., 12.), s(4., 12.), s(4., 12.), s(4., 12.)]
    );
}
#[test]
fn pause_accepts_new_value() {
    let (term, _addon, events) = setup();
    for p in ["4;1;12", "4;4;25", "4;4;123"] {
        term.osc(9, p);
    }
    assert_eq!(*events.borrow(), [s(1., 12.), s(4., 25.), s(4., 100.)]);
}
#[test]
fn invalid_sequences_do_not_emit() {
    let (term, _addon, events) = setup();
    term.osc(9, "4;5;12");
    term.osc(9, "4;1; 123xxxx");
    assert!(events.borrow().is_empty());
}
#[test]
fn fallback_malformed_parts_dispose_and_no_ris_reset() {
    let (term, addon, events) = setup();
    assert!(!term.osc(9, "5;1;12"));
    assert!(term.osc(9, "4;1;2;3"));
    assert!(events.borrow().is_empty());
    term.osc(9, "4;1;12");
    assert!(!term.ris());
    assert_eq!(addon.progress(), s(1., 12.));
    addon.dispose();
    assert!(!term.osc(9, "4;1;99"));
    assert_eq!(addon.progress(), s(1., 12.));
    assert_eq!(events.borrow().len(), 1);
}
#[test]
fn public_numeric_setter_matches_javascript_and_event_lifecycle() {
    let addon = ProgressAddon::new();
    assert!(addon.on_change().is_none());
    assert!(addon.set_progress(s(1.5, 50.5)).unwrap());
    assert_eq!(addon.progress(), s(1.5, 50.5));
    let terminal = Rc::new(FixtureTerminal::default());
    addon.activate(terminal);
    assert!(!addon.set_progress(s(-1., 0.)).unwrap());
    assert!(!addon.set_progress(s(5., 0.)).unwrap());
    assert!(addon.set_progress(s(2., f64::INFINITY)).unwrap());
    assert_eq!(addon.progress(), s(2., 100.));
    assert!(addon.set_progress(s(2., f64::NEG_INFINITY)).unwrap());
    assert_eq!(addon.progress(), s(2., 0.));
    assert!(addon.set_progress(s(f64::NAN, f64::NAN)).unwrap());
    assert!(addon.progress().state.is_nan());
    assert!(addon.progress().value.is_nan());
}
