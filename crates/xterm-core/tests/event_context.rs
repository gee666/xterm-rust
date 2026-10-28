//! Shared conversion values live for one synchronous event dispatch only.
use std::cell::{Cell, RefCell};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::{Rc, Weak};
use xterm_core::event::{DispatchKey, EmitError, Emitter, Event};

#[test]
fn listeners_share_mutable_value_and_no_root_survives_fire() {
    let emitter = Emitter::<i32>::new();
    let key = DispatchKey::<RefCell<Vec<i32>>>::new();
    let first_key = key.clone();
    type Payload = RefCell<Vec<i32>>;
    let weak: Rc<RefCell<Option<Weak<Payload>>>> = Rc::default();
    let output = Rc::clone(&weak);
    let creates = Rc::new(Cell::new(0));
    let calls = Rc::clone(&creates);
    emitter
        .event()
        .try_subscribe_with_context(move |value, context| {
            let payload = context.memoize_with(&first_key, || {
                calls.set(calls.get() + 1);
                RefCell::new(vec![*value])
            })?;
            payload.borrow_mut().push(10);
            *output.borrow_mut() = Some(Rc::downgrade(&payload));
            Ok(())
        });
    emitter
        .event()
        .try_subscribe_with_context(move |value, context| {
            let payload = context.memoize_with(&key, || panic!("converted twice"))?;
            assert_eq!(*payload.borrow(), [*value, 10]);
            Ok(())
        });
    emitter.fire(&3).unwrap();
    assert!(weak.borrow().as_ref().unwrap().upgrade().is_none());
    emitter.fire(&4).unwrap();
    assert_eq!(creates.get(), 2);
}

#[test]
fn nested_fires_get_independent_contexts_and_restore_outer_value() {
    let emitter = Rc::new(Emitter::<i32>::new());
    let key = DispatchKey::<Cell<i32>>::new();
    let first_key = key.clone();
    let nested = Rc::downgrade(&emitter);
    emitter
        .event()
        .try_subscribe_with_context(move |value, context| {
            let payload = context.memoize_with(&first_key, || Cell::new(*value))?;
            payload.set(*value + 100);
            if *value == 1 {
                nested.upgrade().unwrap().fire(&2)?;
            }
            let retained = context.memoize_with(&first_key, || panic!("lost outer value"))?;
            assert!(Rc::ptr_eq(&payload, &retained));
            Ok(())
        });
    let observations = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&observations);
    emitter
        .event()
        .try_subscribe_with_context(move |_, context| {
            output
                .borrow_mut()
                .push(context.memoize_with(&key, || panic!("missing"))?.get());
            Ok(())
        });
    emitter.fire(&1).unwrap();
    assert_eq!(*observations.borrow(), [102, 101]);
}

#[test]
fn different_converter_keys_do_not_alias_same_result_type() {
    let emitter = Emitter::<()>::new();
    let first = DispatchKey::<String>::new();
    let second = DispatchKey::<String>::new();
    emitter
        .event()
        .try_subscribe_with_context(move |_, context| {
            let a = context.memoize_with(&first, || "first".to_string())?;
            let b = context.memoize_with(&second, || "second".to_string())?;
            assert_eq!(&*a, "first");
            assert_eq!(&*b, "second");
            assert!(!Rc::ptr_eq(&a, &b));
            Ok(())
        });
    emitter.fire(&()).unwrap();
}

struct DropValue {
    id: u32,
    drops: Rc<Cell<usize>>,
}
impl Drop for DropValue {
    fn drop(&mut self) {
        self.drops.set(self.drops.get() + 1);
    }
}

#[test]
fn roots_drop_on_errors_and_panics_without_poisoning_the_emitter() {
    for panic in [false, true] {
        let emitter = Emitter::<()>::new();
        let key = DispatchKey::<DropValue>::new();
        let drops = Rc::new(Cell::new(0));
        let output = Rc::clone(&drops);
        let handle = emitter
            .event()
            .try_subscribe_with_context(move |_, context| {
                context.memoize_with(&key, || DropValue {
                    id: 1,
                    drops: Rc::clone(&output),
                })?;
                assert!(!panic, "listener panic");
                Err(EmitError::ListenerArrayCleared)
            });
        let result = catch_unwind(AssertUnwindSafe(|| emitter.fire(&())));
        if panic {
            assert!(result.is_err());
        } else {
            assert_eq!(result.unwrap(), Err(EmitError::ListenerArrayCleared));
        }
        assert_eq!(drops.get(), 1);
        handle.dispose();
        emitter.fire(&()).unwrap();
    }
}

#[test]
fn any_and_forward_keep_the_source_context() {
    let source = Emitter::<i32>::new();
    let other = Emitter::<i32>::new();
    let target = Rc::new(Emitter::<i32>::new());
    let key = DispatchKey::<Cell<i32>>::new();
    let target_key = key.clone();
    target
        .event()
        .try_subscribe_with_context(move |value, context| {
            let payload = context.memoize_with(&target_key, || Cell::new(*value))?;
            payload.set(payload.get() + 1);
            Ok(())
        });
    Event::any([source.event(), other.event()]).forward(target);
    source
        .event()
        .try_subscribe_with_context(move |value, context| {
            assert_eq!(
                context
                    .memoize_with(&key, || panic!("forward lost context"))?
                    .get(),
                *value + 1
            );
            Ok(())
        });
    source.fire(&7).unwrap();
    other.fire(&8).unwrap();
}

#[test]
fn map_and_filter_map_create_fresh_output_contexts() {
    let source = Emitter::<i32>::new();
    let calls = Rc::new(Cell::new(0));
    let mapper = Rc::clone(&calls);
    let mapped = source
        .event()
        .filter_map(move |value| {
            if *value == 0 {
                return None;
            }
            mapper.set(mapper.get() + 1);
            Some(format!("{value}:{}", mapper.get()))
        })
        .map(Clone::clone);
    let key = DispatchKey::<String>::new();
    let observations = Rc::new(RefCell::new(Vec::new()));
    for _ in 0..2 {
        let key = key.clone();
        let output = Rc::clone(&observations);
        mapped.try_subscribe_with_context(move |value, context| {
            output
                .borrow_mut()
                .push((*context.memoize_with(&key, || value.clone())?).clone());
            Ok(())
        });
    }
    source.fire(&0).unwrap();
    assert_eq!(calls.get(), 0);
    source.fire(&3).unwrap();
    assert_eq!(*observations.borrow(), ["3:1", "3:2"]);
}

#[test]
fn failed_conversion_is_not_cached_and_keeps_original_cause() {
    let emitter = Emitter::<()>::new();
    let key = DispatchKey::<String>::new();
    let error = EmitError::callback(std::io::Error::other("conversion failed"));
    emitter
        .event()
        .try_subscribe_with_context(move |_, context| {
            let result = context.try_memoize_with(&key, || Err(error.clone()));
            assert_eq!(result.unwrap_err(), error);
            let recovered = context.memoize_with(&key, || "recovered".to_string())?;
            assert_eq!(&*recovered, "recovered");
            Ok(())
        });
    emitter.fire(&()).unwrap();
}

#[test]
fn first_completed_recursive_conversion_wins_without_a_borrow_conflict() {
    let emitter = Emitter::<()>::new();
    let key = DispatchKey::<DropValue>::new();
    let drops = Rc::new(Cell::new(0));
    let count = Rc::clone(&drops);
    emitter
        .event()
        .try_subscribe_with_context(move |_, context| {
            let retained = context.try_memoize_with(&key, || {
                let first = context.memoize_with(&key, || DropValue {
                    id: 1,
                    drops: Rc::clone(&count),
                })?;
                assert_eq!(first.id, 1);
                Ok(DropValue {
                    id: 2,
                    drops: Rc::clone(&count),
                })
            })?;
            assert_eq!(retained.id, 1);
            assert_eq!(count.get(), 1);
            Ok(())
        });
    emitter.fire(&()).unwrap();
    assert_eq!(drops.get(), 2);
}
