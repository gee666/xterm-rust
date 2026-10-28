// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
#[path = "support/cleanup_factory.rs"]
mod support;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};
use support::{factory, resource, service};
use xterm_core::{
    buffer::{
        buffer_set::{BufferRef, CleanupFactory, CleanupFactoryError, CleanupFactoryHandle},
        BufferError,
    },
    options::{CoercionError, OptionObject, OptionValue},
};

#[test]
fn installation_includes_owner_values_left_by_nested_reset() {
    let service = service();
    let weak = Rc::downgrade(&service);
    let intermediate: Rc<RefCell<Option<Weak<RefCell<xterm_core::buffer::Buffer>>>>> =
        Rc::new(RefCell::new(None));
    let capture = Rc::clone(&intermediate);
    service
        .buffer()
        .borrow_mut()
        .add_marker(0)
        .on_dispose()
        .subscribe(move |_| {
            let service = weak.upgrade().unwrap();
            *capture.borrow_mut() = Some(Rc::downgrade(&service.buffers.normal()));
            service.buffers.reset().unwrap();
        });
    service.buffers.reset().unwrap();
    let retained = intermediate.borrow().as_ref().unwrap().upgrade().unwrap();
    assert!(!Rc::ptr_eq(&retained, &service.buffers.normal()));
    let touched: Rc<RefCell<Vec<Weak<RefCell<xterm_core::buffer::Buffer>>>>> =
        Rc::new(RefCell::new(Vec::new()));
    let seen = Rc::clone(&touched);
    let host: CleanupFactory = Rc::new(move |buffer| {
        seen.borrow_mut().push(Rc::downgrade(buffer));
        Ok(Box::new(()))
    });
    let _handle = service.install_cleanup_factory(host).unwrap();
    assert_eq!(touched.borrow().len(), 3);
    assert!(touched
        .borrow()
        .iter()
        .any(|weak| Rc::ptr_eq(&weak.upgrade().unwrap(), &retained)));
}
#[test]
fn installation_includes_an_old_active_buffer_after_constructor_failure() {
    let service = service();
    let old = service.buffer();
    let options = Rc::clone(&old.borrow().options);
    let calls = Cell::new(0);
    options
        .set_raw(
            "tabStopWidth",
            OptionValue::Object(OptionObject::new((), move |_| {
                let call = calls.get();
                calls.set(call + 1);
                if call == 1 {
                    Err(CoercionError::new("alt construction failed"))
                } else {
                    Ok(OptionValue::Number(8.0))
                }
            })),
        )
        .unwrap();
    assert!(service.buffers.reset().is_err());
    assert!(!Rc::ptr_eq(&old, &service.buffers.normal()));
    assert!(Rc::ptr_eq(&old, &service.buffer()));
    options
        .set_raw("tabStopWidth", OptionValue::Number(8.0))
        .unwrap();
    let calls = Rc::new(Cell::new(0));
    let drops = Rc::new(RefCell::new(Vec::new()));
    let _handle = service
        .install_cleanup_factory(factory(Rc::clone(&calls), drops))
        .unwrap();
    assert_eq!(calls.get(), 3);
}
#[test]
fn creation_hook_runs_before_raw_assignment_and_viewport_filling() {
    let service = service();
    let weak = Rc::downgrade(&service);
    let calls = Rc::new(Cell::new(0));
    let seen = Rc::clone(&calls);
    let host: CleanupFactory = Rc::new(move |buffer| {
        let call = seen.get();
        seen.set(call + 1);
        if call >= 2 {
            let service = weak.upgrade().unwrap();
            assert!(buffer.borrow().lines.is_empty());
            assert!(!Rc::ptr_eq(buffer, &service.buffers.normal()));
            assert!(!Rc::ptr_eq(buffer, &service.buffers.alt()));
        }
        Ok(Box::new(()))
    });
    let _handle = service.install_cleanup_factory(host).unwrap();
    service.reset().unwrap();
    assert_eq!(calls.get(), 4);
}
#[test]
fn detaching_removes_the_factory_before_adapter_destruction_reenters() {
    let service = service();
    let weak = Rc::downgrade(&service);
    let calls = Rc::new(Cell::new(0));
    let seen = Rc::clone(&calls);
    let drops = Rc::new(RefCell::new(Vec::new()));
    let released = Rc::clone(&drops);
    let nested = Rc::new(Cell::new(false));
    let host: CleanupFactory = Rc::new(move |_| {
        let id = seen.get();
        seen.set(id + 1);
        let weak = weak.clone();
        let nested = Rc::clone(&nested);
        Ok(resource(
            id,
            Rc::clone(&released),
            Some(Box::new(move || {
                if !nested.replace(true) {
                    weak.upgrade().unwrap().reset().unwrap();
                }
            })),
        ))
    });
    let handle = service.install_cleanup_factory(host).unwrap();
    handle.dispose();
    assert_eq!(calls.get(), 2);
    assert_eq!(drops.borrow().len(), 2);
}
#[test]
fn detaching_an_old_registration_does_not_remove_a_reentrant_replacement() {
    let service = service();
    let weak = Rc::downgrade(&service);
    let replacements: Rc<RefCell<Option<CleanupFactoryHandle>>> = Rc::new(RefCell::new(None));
    let saved = Rc::clone(&replacements);
    let calls = Rc::new(Cell::new(0));
    let seen = Rc::clone(&calls);
    let drops = Rc::new(RefCell::new(Vec::new()));
    let released = Rc::clone(&drops);
    let host: CleanupFactory = Rc::new(move |_| {
        let id = seen.get();
        seen.set(id + 1);
        let callback = if id == 0 {
            let weak = weak.clone();
            let saved = Rc::clone(&saved);
            Some(Box::new(move || {
                let calls = Rc::new(Cell::new(0));
                let drops = Rc::new(RefCell::new(Vec::new()));
                *saved.borrow_mut() = Some(
                    weak.upgrade()
                        .unwrap()
                        .install_cleanup_factory(factory(calls, drops))
                        .unwrap(),
                );
            }) as Box<dyn FnOnce()>)
        } else {
            None
        };
        Ok(resource(id, Rc::clone(&released), callback))
    });
    let first = service.install_cleanup_factory(host).unwrap();
    first.dispose();
    assert!(replacements.borrow().is_some());
    let extra = service
        .install_cleanup_factory(factory(
            Rc::new(Cell::new(0)),
            Rc::new(RefCell::new(Vec::new())),
        ))
        .unwrap_err();
    let BufferError::Dispatch(error) = extra else {
        panic!("replacement remains installed")
    };
    assert_eq!(
        error.callback_ref::<CleanupFactoryError>(),
        Some(&CleanupFactoryError::AlreadyInstalled)
    );
    service.reset().unwrap();
}
#[test]
fn a_new_buffer_is_not_published_if_its_factory_closes_during_creation() {
    let service = service();
    let old = service.buffer();
    let calls = Rc::new(Cell::new(0));
    let seen = Rc::clone(&calls);
    let handle: Rc<RefCell<Option<CleanupFactoryHandle>>> = Rc::new(RefCell::new(None));
    let weak = Rc::downgrade(&handle);
    let host: CleanupFactory = Rc::new(move |_: &BufferRef| {
        let call = seen.get();
        seen.set(call + 1);
        if call >= 2 {
            weak.upgrade()
                .unwrap()
                .borrow_mut()
                .take()
                .unwrap()
                .dispose();
        }
        Ok(Box::new(()))
    });
    *handle.borrow_mut() = Some(service.install_cleanup_factory(host).unwrap());
    let BufferError::Dispatch(error) = service.reset().unwrap_err() else {
        panic!("detached factory")
    };
    assert_eq!(
        error.callback_ref::<CleanupFactoryError>(),
        Some(&CleanupFactoryError::DetachedDuringCallback)
    );
    assert!(Rc::ptr_eq(&old, &service.buffer()));
    assert!(Rc::ptr_eq(&old, &service.buffers.normal()));
    service.reset().unwrap();
}
