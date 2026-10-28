//! Explicit-only stores are used by EventUtils.any subscription handles.
use std::cell::Cell;
use std::rc::Rc;
use xterm_core::event::{Emitter, Event};
use xterm_core::lifecycle::{to_disposable, Disposable, DisposableStore};

#[test]
fn native_default_and_explicit_store_have_different_drop_boundaries() {
    let count = Rc::new(Cell::new(0));
    for explicit in [false, true] {
        let store = if explicit {
            DisposableStore::new_explicit()
        } else {
            DisposableStore::new()
        };
        let output = Rc::clone(&count);
        store.add(to_disposable(move || output.set(output.get() + 1)));
        drop(store);
    }
    assert_eq!(count.get(), 1);
}

#[test]
fn explicit_disposal_is_idempotent_and_deduplicates_handle_identity() {
    let count = Rc::new(Cell::new(0));
    let output = Rc::clone(&count);
    let handle = to_disposable(move || output.set(output.get() + 1));
    let store = DisposableStore::new_explicit();
    assert!(Rc::ptr_eq(&store.add(Rc::clone(&handle)), &handle));
    store.add(handle);
    store.dispose();
    store.dispose();
    drop(store);
    assert_eq!(count.get(), 1);
}

#[test]
fn discarding_any_handle_keeps_listeners_without_leaking_capture() {
    let source = Emitter::<i32>::new();
    let capture = Rc::new(Cell::new(0));
    let weak = Rc::downgrade(&capture);
    Event::any([source.event()]).subscribe(move |_| capture.set(capture.get() + 1));
    source.fire(&1).unwrap();
    assert_eq!(weak.upgrade().unwrap().get(), 1);
    source.dispose();
    assert!(weak.upgrade().is_none());
}

#[test]
fn dropped_explicit_store_releases_handles_without_executing_them() {
    let capture = Rc::new(Cell::new(0));
    let weak = Rc::downgrade(&capture);
    let store = DisposableStore::new_explicit();
    store.add(to_disposable(move || capture.set(1)));
    drop(store);
    assert!(weak.upgrade().is_none());
}
