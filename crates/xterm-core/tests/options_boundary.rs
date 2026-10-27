// Copyright (c) 2019 The xterm.js authors. All rights reserved.
// Licensed under the MIT license.
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use xterm_core::lifecycle::DisposableHandle;
use xterm_core::options::{
    CoercionError, CoercionHint, OptionError, OptionObject, OptionValue as V, OptionsService,
    WindowsPty,
};

#[test]
fn numeric_then_string_hint_and_failure_atomicity() {
    let service = OptionsService::default();
    let hints = Rc::new(RefCell::new(Vec::new()));
    let output = hints.clone();
    let object = V::Object(OptionObject::new((), move |hint| {
        output.borrow_mut().push(hint);
        Ok(if hint == CoercionHint::Number {
            V::Number(0.0)
        } else {
            V::from("zero")
        })
    }));
    let error = service.set("scrollSensitivity", object).unwrap_err();
    assert_eq!(
        error.to_string(),
        "scrollSensitivity cannot be less than or equal to 0, value: zero"
    );
    assert_eq!(
        *hints.borrow(),
        [CoercionHint::Number, CoercionHint::String]
    );
    assert_eq!(service.get("scrollSensitivity").unwrap(), V::Number(1.0));
}

#[test]
fn object_conversion_failure_is_not_nan_or_success() {
    let service = OptionsService::default();
    let invalid = V::Object(OptionObject::new((), |_| Ok(V::object(()))));
    let error = service.set("cursorWidth", invalid.clone()).unwrap_err();
    assert!(matches!(error, OptionError::Coercion(_)));
    assert_eq!(
        error.to_string(),
        "Cannot convert object to primitive value"
    );
    let constructed = OptionsService::new(false, [("cursorWidth", invalid.clone())]);
    assert_eq!(constructed.diagnostics(), [error]);
    assert_eq!(constructed.get("cursorWidth").unwrap(), V::Number(1.0));
    // Unsanitized fields retain objects without attempting ToPrimitive.
    service.set("fontSize", invalid.clone()).unwrap();
    assert_eq!(service.get("fontSize").unwrap(), invalid);
}

#[test]
fn host_exception_text_is_lossless() {
    let service = OptionsService::default();
    let message = vec![0xd800];
    let output = message.clone();
    let value = V::Object(OptionObject::new((), move |_| {
        Err(CoercionError(output.clone()))
    }));
    let error = service.set("lineHeight", value).unwrap_err();
    assert_eq!(error.message_utf16(), message);
}

#[test]
fn structured_defaults_and_typed_pty_have_properties() {
    let service = OptionsService::default();
    let V::Object(scrollbar) = service.get("scrollbar").unwrap() else {
        panic!()
    };
    assert_eq!(
        scrollbar.get_property("showScrollbar").unwrap(),
        V::Bool(true)
    );
    assert_eq!(scrollbar.get_property("other").unwrap(), V::Undefined);
    let pty = V::from(WindowsPty {
        backend: Some("conpty".into()),
        build_number: Some(0.0),
    });
    let V::Object(pty) = pty else { panic!() };
    assert_eq!(pty.get_property("backend").unwrap(), V::from("conpty"));
    assert_eq!(pty.get_property("buildNumber").unwrap(), V::Number(0.0));
    assert_eq!(pty.get_property("unknown").unwrap(), V::Undefined);
    let value = V::property_map([("kittyKeyboard", V::Bool(true))]);
    let V::Object(value) = value else { panic!() };
    assert_eq!(value.get_property("kittyKeyboard").unwrap(), V::Bool(true));
}

#[test]
fn property_getters_forward_host_errors_and_mutations() {
    let payload = Rc::new(Cell::new(false));
    let object = OptionObject::with_properties(
        payload.clone(),
        |_| Ok(V::from("object")),
        |payload, key| {
            if key == "bad" {
                return Err(CoercionError::new("getter failed"));
            }
            Ok(V::Bool(payload.get()))
        },
    );
    assert_eq!(object.get_property("flag").unwrap(), V::Bool(false));
    payload.set(true);
    assert_eq!(object.get_property("flag").unwrap(), V::Bool(true));
    assert_eq!(
        object.get_property("bad").unwrap_err().to_string(),
        "getter failed"
    );
}

#[test]
fn fallible_listener_commits_before_throw_and_stops_traversal() {
    let service = OptionsService::default();
    service
        .try_on_specific_option_change(
            "rows",
            |_| Err(CoercionError::new("listener failed").into()),
        )
        .unwrap();
    let called = Rc::new(Cell::new(false));
    let output = called.clone();
    service
        .on_option_change()
        .subscribe(move |_| output.set(true));
    let error = service.set("rows", V::Number(30.0)).unwrap_err();
    assert_eq!(error.to_string(), "listener failed");
    assert_eq!(service.get("rows").unwrap(), V::Number(30.0));
    assert!(!called.get());
    service.set("cols", V::Number(90.0)).unwrap();
    assert!(called.get());
}

#[test]
fn nested_listener_errors_keep_original_error_and_clear_frames() {
    let service = Rc::new(OptionsService::default());
    let weak = Rc::downgrade(&service);
    service.try_on_option_change(move |key| {
        if key == "cols" {
            weak.upgrade().unwrap().set("rows", V::Number(30.0))?;
        }
        Ok(())
    });
    service
        .try_on_specific_option_change("rows", |_| Err(CoercionError::new("inner").into()))
        .unwrap();
    assert_eq!(
        service
            .set("cols", V::Number(90.0))
            .unwrap_err()
            .to_string(),
        "inner"
    );
    assert_eq!(service.get("cols").unwrap(), V::Number(90.0));
    assert_eq!(service.get("rows").unwrap(), V::Number(30.0));
    service.set("fontSize", V::Number(16.0)).unwrap();
}

#[test]
fn rust_unwind_removes_dispatch_frame_without_swallowing_panic() {
    let service = OptionsService::default();
    let once = Cell::new(true);
    service
        .on_option_change()
        .subscribe(move |_| assert!(!once.replace(false), "first dispatch"));
    assert!(std::panic::catch_unwind(std::panic::AssertUnwindSafe(
        || service.set("rows", V::Number(30.0))
    ))
    .is_err());
    service.set("rows", V::Number(31.0)).unwrap();
}

#[test]
fn subscription_mutation_uses_original_event_snapshot() {
    let service = Rc::new(OptionsService::default());
    let second: Rc<RefCell<Option<DisposableHandle>>> = Rc::new(RefCell::new(None));
    let remove = second.clone();
    let weak = Rc::downgrade(&service);
    let log = Rc::new(RefCell::new(Vec::new()));
    let output = log.clone();
    let once = Cell::new(true);
    service.on_option_change().subscribe(move |_| {
        output.borrow_mut().push("first");
        if once.replace(false) {
            remove.borrow().as_ref().unwrap().dispose();
            let output = output.clone();
            weak.upgrade()
                .unwrap()
                .on_option_change()
                .subscribe(move |_| output.borrow_mut().push("late"));
        }
    });
    let output = log.clone();
    *second.borrow_mut() = Some(
        service
            .on_option_change()
            .subscribe(move |_| output.borrow_mut().push("second")),
    );
    service.set("rows", V::Number(30.0)).unwrap();
    service.set("rows", V::Number(31.0)).unwrap();
    assert_eq!(*log.borrow(), ["first", "second", "first", "late"]);
}
