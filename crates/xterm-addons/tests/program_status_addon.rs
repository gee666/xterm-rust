// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Registration-fixture ports of addon assertions.
// Parser/DA1/RIS terminal reset and browser execution require separate live tests.
#[path = "support/status.rs"]
pub mod support;
use std::cell::RefCell;
use std::rc::Rc;
use support::FixtureTerminal;
use xterm_addons::program_status::{ProgramStatusAddon, StatusRecord, StatusState};
use xterm_core::lifecycle::Disposable;

type Changes = Rc<RefCell<Vec<Rc<[StatusRecord]>>>>;
fn setup() -> (Rc<FixtureTerminal>, ProgramStatusAddon, Changes) {
    let terminal = Rc::new(FixtureTerminal::default());
    let addon = ProgramStatusAddon::new();
    addon.activate(terminal.clone()).unwrap();
    let changes = Rc::new(RefCell::new(Vec::new()));
    let capture = changes.clone();
    addon
        .on_change()
        .subscribe(move |records| capture.borrow_mut().push(records.clone()));
    (terminal, addon, changes)
}
#[test]
fn keeps_reported_records_and_exact_snapshot_identity() {
    let (term, addon, changes) = setup();
    term.osc(7501, "state=working:app=cargo:msg=QnVpbGRpbmc=");
    let snapshot = addon.records();
    let record = &snapshot[0];
    assert_eq!(record.state, StatusState::Working);
    assert_eq!(record.app.as_deref(), Some("cargo"));
    assert_eq!(record.effective_app.as_deref(), Some("cargo"));
    assert_eq!(record.message.as_deref(), Some("Building"));
    assert_eq!(changes.borrow().len(), 1);
    assert!(Rc::ptr_eq(&changes.borrow()[0], &snapshot));
}
#[test]
fn removes_records_with_clear_report() {
    let (term, addon, changes) = setup();
    for payload in [
        "state=working:id=build",
        "state=working:id=build/test",
        "state=done:id=deploy",
        "state=clear:id=build",
    ] {
        term.osc(7501, payload);
    }
    assert_eq!(
        addon
            .records()
            .iter()
            .map(|r| r.id.as_str())
            .collect::<Vec<_>>(),
        ["deploy"]
    );
    assert_eq!(changes.borrow().len(), 4);
}
#[test]
fn rejected_and_noop_reports_fire_nothing() {
    let (term, addon, changes) = setup();
    for payload in [
        "state=paused",
        "state=clear:id=a//b",
        "state=clear:id=missing",
    ] {
        assert!(term.osc(7501, payload));
    }
    assert!(addon.records().is_empty());
    assert!(changes.borrow().is_empty());
}
#[test]
fn answers_exact_query_only_and_respects_disabled_stdin() {
    let (term, _addon, _) = setup();
    for data in ["?", "?", " ?", "??"] {
        term.osc(7501, data);
    }
    assert_eq!(
        *term.replies.borrow(),
        ["\x1b]7501;?\x1b\\", "\x1b]7501;?\x1b\\"]
    );
    term.disable_stdin.set(true);
    term.osc(7501, "?");
    assert_eq!(term.replies.borrow().len(), 2);
}
#[test]
fn ris_clears_and_delegates_to_terminal_core() {
    let (term, addon, changes) = setup();
    term.osc(7501, "state=working");
    assert!(!term.ris());
    assert!(addon.records().is_empty());
    assert_eq!(changes.borrow().len(), 2);
    term.osc(7501, "state=done");
    assert_eq!(addon.records()[0].state, StatusState::Done);
}
#[test]
fn clear_removes_only_id_and_descendants() {
    let (term, addon, changes) = setup();
    for id in ["a", "a/b", "ab"] {
        term.osc(7501, &format!("state=working:id={id}"));
    }
    addon.clear("a").unwrap();
    assert_eq!(addon.records()[0].id, "ab");
    assert_eq!(changes.borrow().len(), 4);
    addon.clear("a").unwrap();
    assert_eq!(changes.borrow().len(), 4);
}
#[test]
fn process_exit_and_prompt_remove_only_active_records() {
    for prompt in [false, true] {
        let (term, addon, changes) = setup();
        for state in ["working", "blocked", "done", "error", "idle"] {
            term.osc(7501, &format!("state={state}:id={state}"));
        }
        if prompt {
            addon.prompt_started().unwrap();
        } else {
            addon.process_exited().unwrap();
        }
        assert_eq!(
            addon
                .records()
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            ["done", "error", "idle"]
        );
        assert_eq!(changes.borrow().len(), 6);
        if prompt {
            addon.prompt_started().unwrap();
        } else {
            addon.process_exited().unwrap();
        }
        assert_eq!(changes.borrow().len(), 6);
    }
}
#[test]
fn dispose_silently_drops_records_and_unregisters_handlers() {
    let (term, addon, changes) = setup();
    term.osc(7501, "state=working");
    addon.dispose();
    assert!(addon.records().is_empty());
    assert!(!term.osc(7501, "state=done"));
    assert!(!term.osc(7501, "?"));
    assert!(!term.ris());
    assert!(term.replies.borrow().is_empty());
    assert_eq!(changes.borrow().len(), 1);
}
#[test]
fn callback_reentrancy_has_no_mutable_store_borrow() {
    let (term, addon, changes) = setup();
    let addon = Rc::new(addon);
    let weak = Rc::downgrade(&addon);
    addon.on_change().try_subscribe(move |records| {
        if !records.is_empty() {
            weak.upgrade().unwrap().clear("")?;
        }
        Ok(())
    });
    term.osc(7501, "state=working");
    assert!(addon.records().is_empty());
    assert_eq!(changes.borrow().len(), 2);
    assert_eq!(changes.borrow()[0].len(), 1);
    assert_eq!(changes.borrow()[1].len(), 0);
}
#[test]
fn dropping_owner_unregisters_but_event_snapshot_remains_valid() {
    let (term, addon, changes) = setup();
    term.osc(7501, "state=done");
    let snapshot = addon.records();
    drop(addon);
    assert!(!term.osc(7501, "?"));
    assert!(!term.osc(7501, "state=idle"));
    assert_eq!(snapshot[0].state, StatusState::Done);
    assert_eq!(changes.borrow().len(), 1);
}
