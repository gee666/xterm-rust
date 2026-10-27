use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::lifecycle::{
    combined_disposable, dispose, dispose_all, dispose_optional, to_disposable, Disposable,
    DisposableHandle, DisposableOwner, DisposableStore, MutableDisposable, NONE,
};

fn counter() -> (Rc<Cell<usize>>, DisposableHandle) {
    let count = Rc::new(Cell::new(0));
    let observed = Rc::clone(&count);
    let handle = to_disposable(move || observed.set(observed.get() + 1));
    (count, handle)
}

#[test]
fn callback_disposal_repeats_and_helpers_preserve_identity() {
    let (calls, handle) = counter();
    let returned = dispose(Rc::clone(&handle));
    assert!(Rc::ptr_eq(&handle, &returned));
    let returned = dispose_optional(Some(Rc::clone(&handle))).unwrap();
    assert!(Rc::ptr_eq(&handle, &returned));
    assert!(dispose_optional(None).is_none());
    assert!(dispose_all(vec![Rc::clone(&handle), handle]).is_empty());
    assert_eq!(calls.get(), 4);
}

#[test]
fn combined_preserves_order_duplicates_and_repeated_calls() {
    let order = Rc::new(RefCell::new(Vec::new()));
    let a_log = Rc::clone(&order);
    let b_log = Rc::clone(&order);
    let a = to_disposable(move || a_log.borrow_mut().push('a'));
    let b = to_disposable(move || b_log.borrow_mut().push('b'));
    let combined = combined_disposable(vec![Rc::clone(&a), b, a]);
    combined.dispose();
    combined.dispose();
    assert_eq!(*order.borrow(), ['a', 'b', 'a', 'a', 'b', 'a']);
    combined_disposable(vec![]).dispose();
    NONE.dispose();
}

#[test]
fn stores_deduplicate_identity_and_clear_allows_reuse() {
    let store = DisposableStore::new();
    let (calls, handle) = counter();
    let returned = store.add(Rc::clone(&handle));
    assert!(Rc::ptr_eq(&returned, &handle));
    store.add(Rc::clone(&handle));
    store.clear();
    assert_eq!(calls.get(), 1);
    assert!(!store.is_disposed());
    store.clear();
    assert_eq!(calls.get(), 1);
    store.add(handle);
    store.dispose();
    store.dispose();
    assert!(store.is_disposed());
    assert_eq!(calls.get(), 2);
    let (late_calls, late_handle) = counter();
    store.add(Rc::clone(&late_handle));
    store.add(late_handle);
    assert_eq!(late_calls.get(), 2);
}

#[test]
fn registration_during_clear_is_visited_in_the_same_pass() {
    let store = Rc::new(DisposableStore::new());
    let weak = Rc::downgrade(&store);
    let (calls, late) = counter();
    store.add(to_disposable(move || {
        weak.upgrade().unwrap().add(Rc::clone(&late));
    }));
    store.clear();
    assert_eq!(calls.get(), 1);
    store.clear();
    assert_eq!(calls.get(), 1);
}

#[test]
fn dispose_closes_store_before_callbacks_and_late_registration_is_immediate() {
    let store = Rc::new(DisposableStore::new());
    let weak = Rc::downgrade(&store);
    let order = Rc::new(RefCell::new(Vec::new()));
    let late_log = Rc::clone(&order);
    let late = to_disposable(move || late_log.borrow_mut().push("late"));
    let callback_log = Rc::clone(&order);
    store.add(to_disposable(move || {
        let store = weak.upgrade().unwrap();
        assert!(store.is_disposed());
        callback_log.borrow_mut().push("before");
        store.dispose();
        store.add(Rc::clone(&late));
        callback_log.borrow_mut().push("after");
    }));
    store.dispose();
    assert_eq!(*order.borrow(), ["before", "late", "after"]);
}

#[test]
fn mutable_replacement_uses_identity_and_disposes_before_assignment() {
    let owner = Rc::new(MutableDisposable::new());
    let weak = Rc::downgrade(&owner);
    let (calls, second) = counter();
    let observed_old = Rc::new(Cell::new(false));
    let observed = Rc::clone(&observed_old);
    let first = to_disposable(move || {
        observed.set(weak.upgrade().unwrap().value().is_some());
    });
    owner.set(Some(Rc::clone(&first)));
    owner.set(Some(Rc::clone(&first)));
    assert!(!observed_old.get());
    assert!(Rc::ptr_eq(&owner.value().unwrap(), &first));
    owner.set(Some(second));
    assert!(observed_old.get());
    assert_eq!(calls.get(), 0);
    owner.clear();
    owner.clear();
    assert_eq!(calls.get(), 1);
    assert!(owner.value().is_none());
}

#[test]
fn mutable_disposal_hides_value_and_ignores_future_assignment() {
    let owner = Rc::new(MutableDisposable::new());
    let weak = Rc::downgrade(&owner);
    let observed_hidden = Rc::new(Cell::new(false));
    let observed = Rc::clone(&observed_hidden);
    owner.set(Some(to_disposable(move || {
        observed.set(weak.upgrade().unwrap().value().is_none());
    })));
    owner.dispose();
    owner.dispose();
    assert!(observed_hidden.get());
    let (calls, late) = counter();
    owner.set(Some(late));
    assert!(owner.value().is_none());
    assert_eq!(calls.get(), 0);
}

#[test]
fn rust_owners_clean_up_at_drop() {
    let (store_calls, store_handle) = counter();
    {
        let store = DisposableStore::new();
        store.add(store_handle);
    }
    assert_eq!(store_calls.get(), 1);
    let (owner_calls, owner_handle) = counter();
    {
        let owner = DisposableOwner::new();
        owner.register(owner_handle);
        owner.dispose();
    }
    assert_eq!(owner_calls.get(), 1);
    let (mutable_calls, mutable_handle) = counter();
    {
        let owner = MutableDisposable::new();
        owner.set(Some(mutable_handle));
    }
    assert_eq!(mutable_calls.get(), 1);
}

#[test]
fn same_trace_as_typescript_oracle() {
    let log = Rc::new(RefCell::new(Vec::new()));
    let record = |name: &'static str| {
        let log = Rc::clone(&log);
        to_disposable(move || log.borrow_mut().push(name))
    };
    let store = DisposableStore::new();
    let a = record("a");
    store.add(Rc::clone(&a));
    store.add(Rc::clone(&a));
    store.add(record("b"));
    store.clear();
    store.add(Rc::clone(&a));
    store.dispose();
    store.dispose();
    store.add(record("late"));
    let mutable = MutableDisposable::new();
    mutable.set(Some(Rc::clone(&a)));
    mutable.set(Some(Rc::clone(&a)));
    mutable.set(Some(record("replacement")));
    mutable.clear();
    mutable.set(Some(record("final")));
    mutable.dispose();
    mutable.set(Some(record("ignored")));
    let group = combined_disposable(vec![Rc::clone(&a), record("group"), a]);
    group.dispose();
    group.dispose();

    // Retained regression from independent review: a nested clear must not reset
    // the outer Set iterator position before a newly added resource is visited.
    let nested_store = Rc::new(DisposableStore::new());
    let weak = Rc::downgrade(&nested_store);
    let entered = Cell::new(false);
    let nested_log = Rc::clone(&log);
    let late = record("nested-b");
    nested_store.add(to_disposable(move || {
        nested_log.borrow_mut().push("nested-a");
        if !entered.replace(true) {
            let store = weak.upgrade().unwrap();
            store.clear();
            store.add(Rc::clone(&late));
        }
    }));
    nested_store.clear();
    nested_store.clear();
    let actual = log.borrow().join("\n") + "\n";
    assert_eq!(actual, include_str!("fixtures/lifecycle-trace.txt"));
}
