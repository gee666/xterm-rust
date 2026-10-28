//! Explicit acceptance cases E1-E3 from docs/acceptance/events-circular-list.md.
use std::cell::{Cell, RefCell};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;
use xterm_core::event::{EmitError, Emitter, Event};
use xterm_core::lifecycle::{Disposable, DisposableStore};

#[test]
fn getter_identity_and_duplicate_subscriptions_are_independent() {
    let emitter = Emitter::<i32>::default();
    let event = emitter.event();
    assert!(event.ptr_eq(&emitter.event()));
    assert!(event.ptr_eq(&event.clone()));
    assert!(!event.ptr_eq(&Emitter::new().event()));
    let count = Rc::new(Cell::new(0));
    let target = Rc::clone(&count);
    let listener = Rc::new(move |_: &i32| target.set(target.get() + 1));
    let first = Rc::clone(&listener);
    let first = event.subscribe(move |value| first(value));
    let second = event.subscribe(move |value| listener(value));
    assert!(!Rc::ptr_eq(&first, &second));
    emitter.fire(&1).unwrap();
    assert_eq!(count.get(), 2);
    first.dispose();
    first.dispose();
    emitter.fire(&2).unwrap();
    assert_eq!(count.get(), 3);
    second.dispose();
    emitter.fire(&3).unwrap();
    assert_eq!(count.get(), 3);
}

#[test]
fn disposed_store_unregisters_immediately_and_handles_retain_identity() {
    let emitter = Emitter::<i32>::new();
    let store = DisposableStore::new();
    store.dispose();
    emitter
        .event()
        .subscribe_in(&store, |_| panic!("disposed store retains listener"));
    emitter.fire(&1).unwrap();
    let mut vector = Vec::new();
    let handle = emitter.event().subscribe_in_vec(&mut vector, |_| {});
    assert!(Rc::ptr_eq(&handle, &vector[0]));
}

#[test]
fn removal_of_future_listener_keeps_current_snapshot_only() {
    let emitter = Emitter::<i32>::new();
    let results = Rc::new(RefCell::new(Vec::new()));
    let later = Rc::new(RefCell::new(
        None::<xterm_core::lifecycle::DisposableHandle>,
    ));
    let target = Rc::clone(&later);
    let output = Rc::clone(&results);
    emitter.event().subscribe(move |_| {
        output.borrow_mut().push(1);
        target.borrow().as_ref().unwrap().dispose();
    });
    let output = Rc::clone(&results);
    *later.borrow_mut() = Some(
        emitter
            .event()
            .subscribe(move |_| output.borrow_mut().push(2)),
    );
    emitter.fire(&1).unwrap();
    emitter.fire(&2).unwrap();
    assert_eq!(*results.borrow(), [1, 2, 1]);
    later.borrow_mut().take();
}

#[test]
fn disposal_in_single_or_final_listener_completes_successfully() {
    for earlier in [false, true] {
        let emitter = Rc::new(Emitter::<i32>::new());
        if earlier {
            emitter.event().subscribe(|_| {});
        }
        let weak = Rc::downgrade(&emitter);
        emitter
            .event()
            .subscribe(move |_| weak.upgrade().unwrap().dispose());
        emitter.fire(&1).unwrap();
        emitter.fire(&2).unwrap();
    }
}

#[test]
fn fallible_callbacks_and_panics_stop_dispatch_without_poisoning_borrows() {
    let emitter = Emitter::<i32>::new();
    let handle = emitter
        .event()
        .try_subscribe(|_| Err(EmitError::ListenerArrayCleared));
    emitter
        .event()
        .subscribe(|_| panic!("later callback reached after error"));
    assert_eq!(emitter.fire(&1), Err(EmitError::ListenerArrayCleared));
    handle.dispose();
    emitter.dispose();
    let emitter = Emitter::<i32>::new();
    let first = Rc::new(Cell::new(true));
    let target = Rc::clone(&first);
    emitter.event().subscribe(move |_| {
        assert!(!target.replace(false), "callback panic");
    });
    let reached = Rc::new(Cell::new(false));
    let target = Rc::clone(&reached);
    emitter.event().subscribe(move |_| target.set(true));
    assert!(catch_unwind(AssertUnwindSafe(|| emitter.fire(&1))).is_err());
    assert!(!reached.get());
    emitter.fire(&2).unwrap();
    assert!(reached.get());
}

#[test]
fn map_runs_per_subscriber_and_stops_after_store_disposal() {
    let emitter = Emitter::<i32>::new();
    let calls = Rc::new(Cell::new(0));
    let target = Rc::clone(&calls);
    let mapped = emitter.event().map(move |value| {
        target.set(target.get() + 1);
        value * 2
    });
    let store = DisposableStore::new();
    let results = Rc::new(RefCell::new(Vec::new()));
    for _ in 0..2 {
        let output = Rc::clone(&results);
        mapped.subscribe_in(&store, move |value| output.borrow_mut().push(*value));
    }
    emitter.fire(&3).unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(*results.borrow(), [6, 6]);
    store.dispose();
    emitter.fire(&4).unwrap();
    assert_eq!(calls.get(), 2);
}

#[test]
fn any_handles_zero_one_and_repeated_sources_and_repeated_disposal() {
    Event::<i32>::any([])
        .subscribe(|_| panic!("empty any event fired"))
        .dispose();
    let emitter = Emitter::new();
    let count = Rc::new(Cell::new(0));
    let target = Rc::clone(&count);
    let single =
        Event::any([emitter.event()]).subscribe(move |_: &i32| target.set(target.get() + 1));
    let target = Rc::clone(&count);
    let multiple = Event::any([emitter.event(), emitter.event()])
        .subscribe(move |_| target.set(target.get() + 1));
    emitter.fire(&1).unwrap();
    assert_eq!(count.get(), 3);
    multiple.dispose();
    multiple.dispose();
    emitter.fire(&2).unwrap();
    assert_eq!(count.get(), 4);
    single.dispose();
    emitter.fire(&3).unwrap();
    assert_eq!(count.get(), 4);
}

#[test]
fn heterogenous_any_has_explicit_payload_erasure() {
    let numeric = Emitter::<i32>::new();
    let textual = Emitter::<String>::new();
    let calls = Rc::new(Cell::new(0));
    let target = Rc::clone(&calls);
    Event::any([numeric.event().map(|_| ()), textual.event().map(|_| ())])
        .subscribe(move |_| target.set(target.get() + 1));
    numeric.fire(&1).unwrap();
    textual.fire(&"text".to_string()).unwrap();
    assert_eq!(calls.get(), 2);
}

#[test]
fn initial_handler_panic_leaves_no_registration() {
    let emitter = Emitter::<i32>::new();
    let calls = Rc::new(Cell::new(0));
    let target = Rc::clone(&calls);
    assert!(catch_unwind(AssertUnwindSafe(|| {
        emitter.event().run_and_subscribe(
            move |_| {
                target.set(target.get() + 1);
                panic!("initial callback failed");
            },
            Some(&1),
        );
    }))
    .is_err());
    emitter.fire(&2).unwrap();
    assert_eq!(calls.get(), 1);
}

#[test]
fn forwarding_preserves_payload_identity_and_vector_disposal() {
    let emitter = Emitter::<Rc<Cell<i32>>>::new();
    let target = Rc::new(Emitter::new());
    let payload = Rc::new(Cell::new(1));
    let identity = Rc::clone(&payload);
    target.event().subscribe(move |value| {
        assert!(Rc::ptr_eq(value, &identity));
        value.set(value.get() + 1);
    });
    let handles = [emitter.event().forward(target)];
    emitter.fire(&payload).unwrap();
    assert_eq!(payload.get(), 2);
    handles[0].dispose();
    emitter.fire(&payload).unwrap();
    assert_eq!(payload.get(), 2);
}

#[test]
fn handles_after_emitter_destruction_release_captured_objects() {
    let captured = Rc::new(Cell::new(0));
    let weak = Rc::downgrade(&captured);
    let handle = {
        let emitter = Emitter::<i32>::new();
        emitter.event().subscribe(move |_| captured.set(1))
    };
    handle.dispose();
    handle.dispose();
    drop(handle);
    assert!(weak.upgrade().is_none());
}

#[test]
fn captured_resource_drop_can_dispose_another_subscription() {
    struct DisposeOnDrop(xterm_core::lifecycle::DisposableHandle);
    impl Drop for DisposeOnDrop {
        fn drop(&mut self) {
            self.0.dispose();
        }
    }
    let emitter = Emitter::<i32>::new();
    let guard = DisposeOnDrop(emitter.event().subscribe(|_| {}));
    emitter.event().subscribe(move |_| {
        let _retained = &guard;
    });
    emitter.dispose();
    emitter.fire(&1).unwrap();
}
