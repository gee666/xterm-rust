use std::cell::{Cell, RefCell};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
use xterm_core::lifecycle::{
    combined_disposable, dispose_all, to_disposable, Disposable, DisposableHandle, DisposableOwner,
    DisposableStore, MutableDisposable,
};

type Log = Rc<RefCell<Vec<&'static str>>>;

fn record(log: &Log, label: &'static str) -> DisposableHandle {
    let log = Rc::clone(log);
    to_disposable(move || log.borrow_mut().push(label))
}

fn fail_once(log: &Log, label: &'static str) -> DisposableHandle {
    let log = Rc::clone(log);
    let fail = Cell::new(true);
    to_disposable(move || {
        log.borrow_mut().push(label);
        if fail.replace(false) {
            panic!("injected lifecycle failure");
        }
    })
}

fn expect_injected_failure(callback: impl FnOnce()) {
    let result = catch_unwind(AssertUnwindSafe(callback));
    let error = result.expect_err("callback must propagate the injected panic");
    assert_eq!(
        error.downcast_ref::<&str>(),
        Some(&"injected lifecycle failure")
    );
}

#[test]
fn callback_failures_match_unchanged_typescript_state_and_order() {
    let log = Log::default();
    let failure = fail_once(&log, "helper-fail");
    let tail = record(&log, "helper-tail");
    expect_injected_failure(|| {
        dispose_all(vec![Rc::clone(&failure), Rc::clone(&tail)]);
    });
    assert_eq!(*log.borrow(), ["helper-fail"]);
    dispose_all(vec![failure, tail]);

    let clear = DisposableStore::new();
    clear.add(record(&log, "clear-before"));
    clear.add(fail_once(&log, "clear-fail"));
    clear.add(record(&log, "clear-after"));
    expect_injected_failure(|| clear.clear());
    assert!(!clear.is_disposed());
    clear.clear();
    clear.clear();

    let closed = DisposableStore::new();
    closed.add(fail_once(&log, "dispose-fail"));
    closed.add(record(&log, "dispose-skipped"));
    expect_injected_failure(|| closed.dispose());
    assert!(closed.is_disposed());
    closed.dispose();
    closed.add(record(&log, "dispose-late"));

    let replace = MutableDisposable::new();
    let old = fail_once(&log, "replace-old");
    let new = record(&log, "replace-new");
    replace.set(Some(Rc::clone(&old)));
    expect_injected_failure(|| replace.set(Some(Rc::clone(&new))));
    assert!(Rc::ptr_eq(&replace.value().unwrap(), &old));
    replace.set(Some(new));
    replace.clear();

    let mutable = MutableDisposable::new();
    mutable.set(Some(fail_once(&log, "mutable-fail")));
    expect_injected_failure(|| mutable.dispose());
    assert!(mutable.value().is_none());
    mutable.set(Some(record(&log, "mutable-ignored")));
    mutable.dispose();
    mutable.dispose();

    let combined = combined_disposable(vec![
        fail_once(&log, "combined-fail"),
        record(&log, "combined-tail"),
    ]);
    expect_injected_failure(|| combined.dispose());
    combined.dispose();
    let actual = log.borrow().join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/lifecycle-failures.txt"));
}

#[test]
fn bare_handles_do_not_dispose_and_stores_use_identity_not_behavior() {
    let calls = Rc::new(Cell::new(0));
    let make = || {
        let calls = Rc::clone(&calls);
        to_disposable(move || calls.set(calls.get() + 1))
    };
    drop(make());
    assert_eq!(calls.get(), 0);
    let first = make();
    let owner = DisposableOwner::new();
    let returned = owner.register(Rc::clone(&first));
    assert!(Rc::ptr_eq(&first, &returned));
    owner.register(first);
    owner.register(make());
    owner.dispose();
    owner.dispose();
    drop(owner);
    assert_eq!(calls.get(), 2);
}

// This test resource intentionally performs cleanup only in its Rust destructor.
// It detects state borrows that accidentally span destruction of a removed Rc.
struct DropCallback(Box<dyn Fn()>);
impl Disposable for DropCallback {
    fn dispose(&self) {}
}
impl Drop for DropCallback {
    fn drop(&mut self) {
        (self.0)();
    }
}

#[test]
fn store_removed_resource_destructor_can_register_without_a_borrow_conflict() {
    let store = Rc::new(DisposableStore::new());
    let weak = Rc::downgrade(&store);
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    store.add(Rc::new(DropCallback(Box::new(move || {
        let observed = Rc::clone(&observed);
        weak.upgrade().unwrap().add(to_disposable(move || {
            observed.set(observed.get() + 1);
        }));
    }))));
    store.clear();
    assert_eq!(calls.get(), 0);
    store.clear();
    assert_eq!(calls.get(), 1);
}

#[test]
fn mutable_removed_resource_destructor_can_replace_without_a_borrow_conflict() {
    let owner = Rc::new(MutableDisposable::new());
    let weak = Rc::downgrade(&owner);
    let calls = Rc::new(Cell::new(0));
    let observed = Rc::clone(&calls);
    owner.set(Some(Rc::new(DropCallback(Box::new(move || {
        let observed = Rc::clone(&observed);
        weak.upgrade().unwrap().set(Some(to_disposable(move || {
            observed.set(observed.get() + 1)
        })));
    })))));
    owner.clear();
    assert_eq!(calls.get(), 0);
    assert!(owner.value().is_some());
    owner.clear();
    assert_eq!(calls.get(), 1);
}
