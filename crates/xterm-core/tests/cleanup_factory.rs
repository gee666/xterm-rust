// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
#[path = "support/cleanup_factory.rs"]
mod support;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use support::{factory, resource, service};
use xterm_core::{
    buffer::{
        buffer_set::{CleanupFactory, CleanupFactoryError},
        BufferError,
    },
    event::EmitError,
};

#[test]
fn factories_attach_to_both_buffers_and_follow_buffer_lifetimes() {
    let service = service();
    let normal = service.buffers.normal();
    let alt = service.buffers.alt();
    let before = Rc::strong_count(&normal);
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(RefCell::new(Vec::new()));
    let handle = service
        .install_cleanup_factory(factory(Rc::clone(&calls), Rc::clone(&drops)))
        .unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(Rc::strong_count(&normal), before);
    service.reset().unwrap();
    assert_eq!(calls.get(), 4);
    assert!(drops.borrow().is_empty());
    drop(normal);
    drop(alt);
    assert_eq!(&*drops.borrow(), &[0, 1]);
    let current = service.buffer();
    let held = current.borrow_mut();
    handle.dispose();
    drop(held);
    assert_eq!(drops.borrow().len(), 4);
    service.reset().unwrap();
    assert_eq!(calls.get(), 4);
}
#[test]
fn a_duplicate_factory_is_rejected_without_replacing_the_first() {
    let service = service();
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(RefCell::new(Vec::new()));
    let first = service
        .install_cleanup_factory(factory(Rc::clone(&calls), Rc::clone(&drops)))
        .unwrap();
    let error = service
        .install_cleanup_factory(factory(Rc::clone(&calls), Rc::clone(&drops)))
        .unwrap_err();
    let BufferError::Dispatch(error) = error else {
        panic!("typed registry error")
    };
    assert_eq!(
        error.callback_ref::<CleanupFactoryError>(),
        Some(&CleanupFactoryError::AlreadyInstalled)
    );
    assert_eq!(calls.get(), 2);
    assert!(drops.borrow().is_empty());
    drop(first);
    let _second = service
        .install_cleanup_factory(factory(Rc::clone(&calls), drops))
        .unwrap();
    assert_eq!(calls.get(), 4);
}
#[test]
fn initial_failure_rolls_back_installed_adapters_and_preserves_the_cause() {
    let service = service();
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(RefCell::new(Vec::new()));
    let cause = EmitError::callback(std::io::Error::other("host failure"));
    let expected = cause.clone();
    let seen = Rc::clone(&calls);
    let released = Rc::clone(&drops);
    let failing: CleanupFactory = Rc::new(move |_| {
        let id = seen.get();
        seen.set(id + 1);
        if id == 1 {
            Err(BufferError::Dispatch(cause.clone()))
        } else {
            Ok(resource(id, Rc::clone(&released), None))
        }
    });
    let BufferError::Dispatch(actual) = service.install_cleanup_factory(failing).unwrap_err()
    else {
        panic!("host cause")
    };
    assert_eq!(actual, expected);
    assert_eq!(&*drops.borrow(), &[0]);
    let _retry = service
        .install_cleanup_factory(factory(calls, drops))
        .unwrap();
}
#[test]
fn a_handle_does_not_retain_buffers_after_the_set_is_dropped() {
    let service = service();
    let buffer = Rc::downgrade(&service.buffer());
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(RefCell::new(Vec::new()));
    let handle = service
        .install_cleanup_factory(factory(calls, Rc::clone(&drops)))
        .unwrap();
    drop(service);
    assert!(buffer.upgrade().is_none());
    assert_eq!(drops.borrow().len(), 2);
    drop(handle);
    assert_eq!(drops.borrow().len(), 2);
}
