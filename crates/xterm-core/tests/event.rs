// Copyright (c) 2024-2026 The xterm.js authors. All rights reserved.
// Licensed under MIT. First seven tests port common/Event.test.ts.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::event::{EmitError, Emitter, Event};
use xterm_core::lifecycle::{Disposable, DisposableHandle, DisposableStore};

#[test]
fn fire_with_zero_listeners() {
    Emitter::<i32>::new().fire(&42).unwrap();
}

#[test]
fn fire_with_one_listener() {
    let emitter = Emitter::new();
    let received = Rc::new(Cell::new(None));
    let output = Rc::clone(&received);
    emitter
        .event()
        .subscribe(move |&value| output.set(Some(value)));
    emitter.fire(&42).unwrap();
    assert_eq!(received.get(), Some(42));
}

#[test]
fn fire_with_receiver_captured_instead_of_this_args() {
    struct Receiver {
        value: Cell<i32>,
    }
    impl Receiver {
        fn handler(&self, value: i32) {
            self.value.set(value);
        }
    }
    let emitter = Emitter::new();
    let receiver = Rc::new(Receiver {
        value: Cell::new(0),
    });
    let target = Rc::clone(&receiver);
    emitter
        .event()
        .subscribe(move |&value| target.handler(value));
    emitter.fire(&42).unwrap();
    assert_eq!(receiver.value.get(), 42);
}

#[test]
fn fire_with_multiple_listeners() {
    let emitter = Emitter::new();
    let results = Rc::new(RefCell::new(Vec::new()));
    for factor in 1..=3 {
        let output = Rc::clone(&results);
        emitter
            .event()
            .subscribe(move |&value| output.borrow_mut().push(value * factor));
    }
    emitter.fire(&10).unwrap();
    assert_eq!(*results.borrow(), [10, 20, 30]);
}

#[test]
fn listener_removal_during_fire() {
    let emitter = Emitter::new();
    let results = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&results);
    emitter
        .event()
        .subscribe(move |_| output.borrow_mut().push("first"));
    let disposable: Rc<RefCell<Option<DisposableHandle>>> = Rc::default();
    let target = Rc::clone(&disposable);
    let output = Rc::clone(&results);
    *disposable.borrow_mut() = Some(emitter.event().subscribe(move |_| {
        output.borrow_mut().push("second");
        target.borrow().as_ref().unwrap().dispose();
    }));
    let output = Rc::clone(&results);
    emitter
        .event()
        .subscribe(move |_| output.borrow_mut().push("third"));
    emitter.fire(&1).unwrap();
    assert_eq!(*results.borrow(), ["first", "second", "third"]);
    disposable.borrow_mut().take();
}

#[test]
fn no_fire_after_dispose() {
    let emitter = Emitter::new();
    let called = Rc::new(Cell::new(false));
    let output = Rc::clone(&called);
    emitter.event().subscribe(move |_| output.set(true));
    emitter.dispose();
    emitter.fire(&42).unwrap();
    assert!(!called.get());
}

#[test]
fn dispose_listener() {
    let emitter = Emitter::new();
    let count = Rc::new(Cell::new(0));
    let output = Rc::clone(&count);
    let disposable = emitter
        .event()
        .subscribe(move |_| output.set(output.get() + 1));
    emitter.fire(&1).unwrap();
    disposable.dispose();
    emitter.fire(&2).unwrap();
    assert_eq!(count.get(), 1);
}

#[test]
fn additions_and_removals_use_snapshot_and_nested_fire_sees_new_list() {
    let emitter = Rc::new(Emitter::new());
    let results = Rc::new(RefCell::new(Vec::new()));
    let second: Rc<RefCell<Option<DisposableHandle>>> = Rc::default();
    let to_remove = Rc::clone(&second);
    let nested = Rc::downgrade(&emitter);
    let output = Rc::clone(&results);
    emitter.event().try_subscribe(move |&value| {
        output.borrow_mut().push((1, value));
        if value == 1 {
            to_remove.borrow().as_ref().unwrap().dispose();
            let inner = Rc::clone(&output);
            let nested = nested.upgrade().unwrap();
            nested
                .event()
                .subscribe(move |&value| inner.borrow_mut().push((3, value)));
            nested.fire(&2)?;
        }
        Ok(())
    });
    let output = Rc::clone(&results);
    *second.borrow_mut() = Some(
        emitter
            .event()
            .subscribe(move |&value| output.borrow_mut().push((2, value))),
    );
    emitter.fire(&1).unwrap();
    assert_eq!(*results.borrow(), [(1, 1), (1, 2), (3, 2), (2, 1)]);
    second.borrow_mut().take();
}

#[test]
fn dispose_during_active_multi_listener_dispatch_returns_upstream_error() {
    let emitter = Rc::new(Emitter::new());
    let weak = Rc::downgrade(&emitter);
    emitter
        .event()
        .subscribe(move |_| weak.upgrade().unwrap().dispose());
    emitter
        .event()
        .subscribe(|_| panic!("upstream never reaches this listener"));
    assert_eq!(emitter.fire(&1), Err(EmitError::ListenerArrayCleared));
    emitter.fire(&1).unwrap();
}

#[test]
fn dispose_after_copying_list_does_not_clear_active_snapshot() {
    let emitter = Rc::new(Emitter::new());
    let weak = Rc::downgrade(&emitter);
    let seen = Rc::new(Cell::new(false));
    emitter.event().subscribe(move |_| {
        let emitter = weak.upgrade().unwrap();
        emitter.event().subscribe(|_| {});
        emitter.dispose();
    });
    let output = Rc::clone(&seen);
    emitter.event().subscribe(move |_| output.set(true));
    emitter.fire(&1).unwrap();
    assert!(seen.get());
}

#[test]
fn registration_after_disposal_is_noop() {
    let emitter = Emitter::new();
    emitter.dispose();
    let handle = emitter
        .event()
        .subscribe(|_: &i32| panic!("disposed emitter fired"));
    handle.dispose();
    handle.dispose();
    emitter.dispose();
    emitter.fire(&42).unwrap();
}

#[test]
fn store_and_array_dispose_registrations() {
    let emitter = Emitter::new();
    let store = DisposableStore::new();
    let mut array = Vec::new();
    let count = Rc::new(Cell::new(0));
    let output = Rc::clone(&count);
    emitter
        .event()
        .subscribe_in(&store, move |_: &i32| output.set(output.get() + 1));
    let output = Rc::clone(&count);
    emitter
        .event()
        .subscribe_in_vec(&mut array, move |_| output.set(output.get() + 1));
    emitter.fire(&1).unwrap();
    assert_eq!(count.get(), 2);
    store.clear();
    array[0].dispose();
    emitter.fire(&2).unwrap();
    assert_eq!(count.get(), 2);
}

#[test]
fn map_any_and_forward_preserve_disposal() {
    let first = Emitter::new();
    let second = Emitter::new();
    let target = Rc::new(Emitter::new());
    let values = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&values);
    target
        .event()
        .subscribe(move |value| output.borrow_mut().push(*value));
    let combined = Event::any([first.event(), second.event()]).map(|value| value * 2);
    let handle = combined.forward(Rc::clone(&target));
    first.fire(&2).unwrap();
    second.fire(&3).unwrap();
    assert_eq!(*values.borrow(), [4, 6]);
    handle.dispose();
    first.fire(&9).unwrap();
    second.fire(&10).unwrap();
    assert_eq!(*values.borrow(), [4, 6]);
}

#[test]
fn run_and_subscribe_initial_value_precedes_registration() {
    let emitter = Rc::new(Emitter::new());
    let values = Rc::new(RefCell::new(Vec::new()));
    let weak = Rc::downgrade(&emitter);
    let output = Rc::clone(&values);
    let handle = emitter.event().run_and_subscribe(
        move |value| {
            output.borrow_mut().push(value.copied());
            if value.is_none() {
                weak.upgrade().unwrap().fire(&5).unwrap();
            }
        },
        None,
    );
    assert_eq!(*values.borrow(), [None]);
    emitter.fire(&6).unwrap();
    assert_eq!(*values.borrow(), [None, Some(6)]);
    handle.dispose();
    let output = Rc::clone(&values);
    emitter.event().run_and_subscribe(
        move |value| output.borrow_mut().push(value.copied()),
        Some(&7),
    );
    assert_eq!(values.borrow().last(), Some(&Some(7)));
}

#[test]
fn forwarded_dispatch_error_propagates() {
    let source = Emitter::new();
    let target = Rc::new(Emitter::new());
    let weak = Rc::downgrade(&target);
    target
        .event()
        .subscribe(move |_: &i32| weak.upgrade().unwrap().dispose());
    target.event().subscribe(|_| panic!("unreachable callback"));
    source.event().forward(target);
    assert_eq!(source.fire(&1), Err(EmitError::ListenerArrayCleared));
}

#[test]
fn retained_event_keeps_emitter_state_alive_without_cycles() {
    let event = {
        let emitter = Emitter::<i32>::new();
        emitter.event()
    };
    let count = Rc::new(Cell::new(0));
    let weak = Rc::downgrade(&count);
    let handle = event.subscribe(move |_| count.set(1));
    assert!(weak.upgrade().is_some());
    handle.dispose();
    drop(handle);
    drop(event);
    assert!(weak.upgrade().is_none());
}
