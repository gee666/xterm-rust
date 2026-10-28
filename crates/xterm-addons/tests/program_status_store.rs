// Copyright (c) 2026 The xterm.js authors. All rights reserved.
// Licensed under the MIT license. Assertion port of ProgramStatus.test.ts store suite.
use std::rc::Rc;
use xterm_addons::program_status::{ProgramStatusStore, StatusKind, StatusRecord, StatusState};
use StatusState::{Blocked, Done, Error, Idle, Working};

fn record(id: &str, state: StatusState) -> StatusRecord {
    StatusRecord::new(id, state)
}
fn app(id: &str, state: StatusState, app: &str) -> StatusRecord {
    let mut r = record(id, state);
    r.app = Some(app.into());
    r
}
fn ids(store: &mut ProgramStatusStore) -> Vec<String> {
    store.records().iter().map(|r| r.id.clone()).collect()
}

#[test]
fn replaces_a_record_whole() {
    let mut store = ProgramStatusStore::new();
    let mut r = record("a", Blocked);
    r.kind = Some(StatusKind::Permission);
    r.message = Some("Allow?".into());
    store.replace(r);
    store.replace(record("a", Working));
    assert_eq!(&*store.records(), &[record("a", Working)]);
}
#[test]
fn orders_records_from_oldest_update_to_newest() {
    let mut store = ProgramStatusStore::new();
    for (id, state) in [("a", Working), ("b", Working), ("a", Done)] {
        store.replace(record(id, state));
    }
    assert_eq!(ids(&mut store), ["b", "a"]);
}
#[test]
fn clears_record_and_descendants() {
    let mut store = ProgramStatusStore::new();
    for id in ["", "build", "build/test", "build/test/unit", "builder"] {
        store.replace(record(id, Working));
    }
    assert!(store.clear("build"));
    assert_eq!(ids(&mut store), ["", "builder"]);
    assert!(!store.clear("build"));
}
#[test]
fn clears_every_record_for_empty_id() {
    let mut store = ProgramStatusStore::new();
    store.replace(record("", Working));
    store.replace(record("a", Working));
    assert!(store.clear(""));
    assert!(store.records().is_empty());
    assert!(!store.clear(""));
}
#[test]
fn removes_working_and_blocked_only() {
    let mut store = ProgramStatusStore::new();
    for (id, state) in [
        ("a", Idle),
        ("b", Working),
        ("c", Done),
        ("d", Blocked),
        ("e", Error),
    ] {
        store.replace(record(id, state));
    }
    assert!(store.remove_active_records());
    assert_eq!(ids(&mut store), ["a", "c", "e"]);
    assert!(!store.remove_active_records());
}
#[test]
fn evicts_least_recently_updated_at_256() {
    let mut store = ProgramStatusStore::new();
    for i in 0..256 {
        store.replace(record(&format!("r{i}"), Idle));
    }
    store.replace(record("r0", Done));
    store.replace(record("new", Idle));
    let ids = ids(&mut store);
    assert_eq!(ids.len(), 256);
    assert_eq!(&ids[..2], ["r2", "r3"]);
    assert_eq!(&ids[254..], ["r0", "new"]);
}
#[test]
fn existing_update_never_evicts_other_record() {
    let mut store = ProgramStatusStore::new();
    for i in 0..256 {
        store.replace(record(&format!("r{i}"), Idle));
    }
    store.replace(record("r100", Done));
    let ids = ids(&mut store);
    assert_eq!(ids.len(), 256);
    assert_eq!(ids[0], "r0");
    assert_eq!(ids[255], "r100");
}
#[test]
fn effective_app_uses_nearest_ancestor_including_root() {
    let mut store = ProgramStatusStore::new();
    store.replace(app("", Idle, "grok"));
    store.replace(app("a", Working, "cargo"));
    for id in ["a/b", "a/b/c", "x/y"] {
        store.replace(record(id, Working));
    }
    let apps: Vec<_> = store
        .records()
        .iter()
        .map(|r| r.effective_app.clone().unwrap())
        .collect();
    assert_eq!(apps, ["grok", "cargo", "cargo", "cargo", "grok"]);
}
#[test]
fn effective_app_is_absent_without_ancestor() {
    let mut store = ProgramStatusStore::new();
    store.replace(record("a/b", Working));
    assert_eq!(&*store.records(), &[record("a/b", Working)]);
}
#[test]
fn effective_app_changes_when_ancestor_changes() {
    let mut store = ProgramStatusStore::new();
    store.replace(app("a", Working, "cargo"));
    store.replace(record("a/b", Working));
    store.replace(app("a", Working, "rustc"));
    assert_eq!(ids(&mut store), ["a/b", "a"]);
    assert!(store
        .records()
        .iter()
        .all(|r| r.effective_app.as_deref() == Some("rustc")));
    store.replace(record("a", Done));
    assert!(store.records().iter().all(|r| r.effective_app.is_none()));
}
#[test]
fn raw_record_identity_survives_unrelated_updates_but_projection_does_not() {
    let mut store = ProgramStatusStore::new();
    store.replace(record("plain", Working));
    store.replace(app("projected", Working, "cargo"));
    let before = store.records();
    store.replace(record("new", Done));
    let after = store.records();
    assert!(Rc::ptr_eq(before[0].identity(), after[0].identity()));
    assert!(!Rc::ptr_eq(before[1].identity(), after[1].identity()));
    store.replace(record("plain", Working));
    let replaced = store.records();
    assert!(!Rc::ptr_eq(before[0].identity(), replaced[2].identity()));
}

#[test]
fn shares_immutable_snapshot_until_real_change() {
    let mut store = ProgramStatusStore::new();
    store.replace(app("a", Working, "cargo"));
    let snapshot = store.records();
    assert!(Rc::ptr_eq(&snapshot, &store.records()));
    store.clear("missing");
    assert!(Rc::ptr_eq(&snapshot, &store.records()));
    store.replace(record("b", Working));
    assert!(!Rc::ptr_eq(&snapshot, &store.records()));
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot[0].id, "a");
}
