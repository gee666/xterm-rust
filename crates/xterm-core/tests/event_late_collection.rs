//! Source Event.ts returns before collection for late emitter subscriptions.
use std::cell::Cell;
use std::rc::Rc;
use xterm_core::event::{Emitter, Event};
use xterm_core::lifecycle::{Disposable, DisposableStore};

#[test]
fn disposed_emitter_does_not_collect_its_noop_handle() {
    let emitter = Emitter::<i32>::new();
    emitter.dispose();
    let mut vector = Vec::new();
    let handle = emitter
        .event()
        .subscribe_in_vec(&mut vector, |_| panic!("late callback"));
    assert!(vector.is_empty());
    handle.dispose();
    handle.dispose();
    let registration = emitter.event().register_with_context(|_, _| Ok(()));
    assert!(!registration.collect);
    registration.handle.dispose();
}

#[test]
fn mapped_and_filtered_late_registration_keeps_vector_empty() {
    let emitter = Emitter::<i32>::new();
    let calls = Rc::new(Cell::new(0));
    let output = Rc::clone(&calls);
    let mapped = emitter.event().map(move |value| {
        output.set(output.get() + 1);
        value + 1
    });
    emitter.dispose();
    let mut vector = Vec::new();
    mapped.subscribe_in_vec(&mut vector, |_| panic!("mapped late callback"));
    mapped
        .filter_map(|value| Some(*value))
        .subscribe_in_vec(&mut vector, |_| panic!("filtered late callback"));
    emitter.fire(&1).unwrap();
    assert_eq!(calls.get(), 0);
    assert!(vector.is_empty());
}

#[test]
fn late_store_registration_differs_from_explicit_store_add() {
    let emitter = Emitter::<i32>::new();
    emitter.dispose();
    let store = DisposableStore::new();
    let handle = emitter.event().subscribe_in(&store, |_| {});
    let weak = Rc::downgrade(&handle);
    drop(handle);
    assert!(weak.upgrade().is_none(), "store must not retain late no-op");
    let explicit = emitter.event().subscribe(|_| {});
    let weak = Rc::downgrade(&explicit);
    store.add(explicit);
    assert!(weak.upgrade().is_some());
    store.clear();
    assert!(weak.upgrade().is_none());
}

#[test]
fn any_over_disposed_sources_still_collects_one_group() {
    let first = Emitter::<i32>::new();
    let second = Emitter::new();
    first.dispose();
    second.dispose();
    let any = Event::any([first.event(), second.event()]);
    let mut vector = Vec::new();
    let handle = any.subscribe_in_vec(&mut vector, |_| {});
    assert_eq!(vector.len(), 1);
    assert!(Rc::ptr_eq(&handle, &vector[0]));
    handle.dispose();
    handle.dispose();
    assert!(any.register_with_context(|_, _| Ok(())).collect);
}

#[test]
fn empty_any_and_map_over_any_still_collect_groups() {
    let empty = Event::<i32>::any([]);
    let mut vector = Vec::new();
    let first = empty.subscribe_in_vec(&mut vector, |_| {});
    let second = empty
        .map(|value| value + 1)
        .subscribe_in_vec(&mut vector, |_| {});
    assert_eq!(vector.len(), 2);
    assert!(!Rc::ptr_eq(&first, &second));
    first.dispose();
    second.dispose();
}

#[test]
fn any_over_mapped_disposed_sources_collects_its_outer_group() {
    let source = Emitter::<i32>::new();
    source.dispose();
    let mapped = source.event().map(|value| value + 1);
    let any = Event::any([mapped]);
    let mut vector = Vec::new();
    any.subscribe_in_vec(&mut vector, |_| {});
    any.map(|value| value + 1)
        .subscribe_in_vec(&mut vector, |_| {});
    assert_eq!(vector.len(), 2);
    for handle in vector {
        handle.dispose();
    }
}
