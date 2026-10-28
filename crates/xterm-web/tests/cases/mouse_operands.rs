use super::{
    getters::{activate, getter, number_object, ErrorTrap},
    Setup,
};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;

#[wasm_bindgen_test]
fn fast_operand_is_obtained_before_left_and_right_coercions() {
    let setup = Setup::new();
    activate(&setup);
    let wheel = setup.wheel();
    let trace = setup.host.trace.clone();
    let (amount, _amount) = number_object(move || {
        trace.borrow_mut().push("coerce:amount".into());
        Ok(1.into())
    });
    let trace = setup.host.trace.clone();
    let (fast, _fast) = number_object(move || {
        trace.borrow_mut().push("coerce:fast".into());
        Ok(2.into())
    });
    let trace = setup.host.trace.clone();
    let (slow, _slow) = number_object(move || {
        trace.borrow_mut().push("coerce:slow".into());
        Ok(3.into())
    });
    *setup.host.fast_value.borrow_mut() = Some(fast);
    *setup.host.slow_value.borrow_mut() = Some(slow);
    let _delta = getter(&wheel, "deltaY", move || Ok(amount.clone()));
    let _alt = getter(&wheel, "altKey", || Ok(true.into()));
    setup.host.trace.borrow_mut().clear();
    setup.dom.element.dispatch_event(&wheel).unwrap();
    let relevant: Vec<_> = setup
        .host
        .trace
        .borrow()
        .iter()
        .filter(|v| v.starts_with("coerce:") || ["host:fast", "host:slow"].contains(&v.as_str()))
        .cloned()
        .collect();
    assert_eq!(
        relevant,
        [
            "host:fast",
            "coerce:amount",
            "coerce:fast",
            "host:slow",
            "coerce:slow",
            "coerce:amount"
        ]
    );
    assert_eq!(setup.host.input.data.borrow().len(), 1);
}

#[wasm_bindgen_test]
fn left_operand_failure_precedes_right_object_coercion() {
    let setup = Setup::new();
    activate(&setup);
    let trap = ErrorTrap::new();
    let wheel = setup.wheel();
    let sentinel: JsValue = js_sys::Object::new().into();
    let thrown = sentinel.clone();
    let (amount, _amount) = number_object(move || Err(thrown.clone()));
    let calls = Rc::new(Cell::new(0));
    let counted = calls.clone();
    let (fast, _fast) = number_object(move || {
        counted.set(counted.get() + 1);
        Err("wrong right failure".into())
    });
    *setup.host.fast_value.borrow_mut() = Some(fast);
    let _delta = getter(&wheel, "deltaY", move || Ok(amount.clone()));
    let _alt = getter(&wheel, "altKey", || Ok(true.into()));
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert_eq!(calls.get(), 0);
    assert_eq!(trap.errors.borrow().len(), 1);
    assert!(js_sys::Object::is(&trap.errors.borrow()[0], &sentinel));
    assert!(!setup.host.reads.borrow().contains(&"slow"));
}

#[wasm_bindgen_test]
fn touch_compound_assignment_captures_accumulator_before_getter() {
    let setup = Setup::new();
    let mut snapshot = setup.host.snapshot.get();
    snapshot.has_scrollback = false;
    snapshot.css_cell_height = 20.0;
    setup.host.snapshot.set(snapshot);
    setup
        .dom
        .screen
        .dispatch_event(&setup.gesture("-xterm-gesturechange", -5.0))
        .unwrap();
    let change = setup.gesture("-xterm-gesturechange", 2.0);
    let start = setup.gesture("-xterm-gesturestart", 0.0);
    let screen = setup.dom.screen.clone();
    let _translation = getter(&change, "translationY", move || {
        screen.dispatch_event(&start)?;
        Ok(2.into())
    });
    setup.dom.screen.dispatch_event(&change).unwrap();
    setup
        .dom
        .screen
        .dispatch_event(&setup.gesture("-xterm-gesturechange", -17.0))
        .unwrap();
    assert_eq!(
        setup.host.input.data.borrow().as_slice(),
        &[(vec![27, 91, 66], true)]
    );
}

#[wasm_bindgen_test]
fn fallback_large_number_is_rejected_before_int32_projection() {
    let setup = Setup::new();
    activate(&setup);
    let movement = setup.pointer("mousemove", 0);
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let _buttons = getter(&movement, "buttons", || Ok(JsValue::UNDEFINED));
    let _button = getter(&movement, "button", move || {
        let n = count.get() + 1;
        count.set(n);
        Ok(if n < 3 { 0.0 } else { 2147483648.0 }.into())
    });
    setup.dom.element.dispatch_event(&movement).unwrap();
    assert_eq!(calls.get(), 3);
    assert!(setup.host.input.data.borrow().is_empty());
    assert!(!setup.host.reads.borrow().contains(&"require-alt"));
}

#[wasm_bindgen_test]
fn fractional_buttons_are_distinct_for_move_debounce() {
    let setup = Setup::new();
    activate(&setup);
    for value in [0.5, 0.6, f64::NAN, f64::NAN] {
        let movement = setup.pointer("mousemove", 0);
        let calls = Rc::new(Cell::new(0));
        let count = calls.clone();
        let _buttons = getter(&movement, "buttons", || Ok(JsValue::UNDEFINED));
        let _button = getter(&movement, "button", move || {
            let n = count.get() + 1;
            count.set(n);
            Ok(if n < 3 { 0.0 } else { value }.into())
        });
        setup.dom.element.dispatch_event(&movement).unwrap();
    }
    assert_eq!(setup.host.input.data.borrow().len(), 4);
}

#[wasm_bindgen_test]
fn bounding_method_failure_is_caught_without_later_reads() {
    let setup = Setup::new();
    activate(&setup);
    let trap = ErrorTrap::new();
    let sentinel: JsValue = js_sys::Object::new().into();
    let thrown = sentinel.clone();
    let method = Closure::wrap(
        Box::new(move || -> Result<JsValue, JsValue> { Err(thrown.clone()) })
            as Box<dyn Fn() -> Result<JsValue, JsValue>>,
    );
    js_sys::Reflect::set(
        setup.dom.screen.as_ref(),
        &"getBoundingClientRect".into(),
        method.as_ref(),
    )
    .unwrap();
    // Do not call Setup::wheel after overriding its geometry method.
    let wheel = super::support::input::event(
        "WheelEvent",
        "wheel",
        &[("deltaY", 1.into()), ("deltaMode", 1.into())],
    );
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert_eq!(trap.errors.borrow().len(), 1);
    assert!(js_sys::Object::is(&trap.errors.borrow()[0], &sentinel));
    assert!(setup.host.reads.borrow().is_empty());
    js_sys::Reflect::delete_property(
        setup.dom.screen.unchecked_ref(),
        &"getBoundingClientRect".into(),
    )
    .unwrap();
}

#[wasm_bindgen_test]
fn geometry_gets_both_operands_before_numeric_coercion() {
    let setup = Setup::new();
    activate(&setup);
    let wheel = setup.wheel();
    let order = Rc::new(std::cell::RefCell::new(Vec::new()));
    let sequence = order.clone();
    let (client, _client) = number_object(move || {
        sequence.borrow_mut().push("client-coerce");
        Ok(4.into())
    });
    let sequence = order.clone();
    let _x = getter(&wheel, "clientX", move || {
        sequence.borrow_mut().push("client-get");
        Ok(client.clone())
    });
    let rect = js_sys::Object::new();
    let sequence = order.clone();
    let _left = getter(rect.as_ref(), "left", move || {
        sequence.borrow_mut().push("left-get");
        Ok(0.into())
    });
    js_sys::Reflect::set(&rect, &"top".into(), &0.into()).unwrap();
    let method =
        Closure::wrap(
            Box::new(move || -> Result<JsValue, JsValue> { Ok(rect.clone().into()) })
                as Box<dyn Fn() -> Result<JsValue, JsValue>>,
        );
    js_sys::Reflect::set(
        setup.dom.screen.as_ref(),
        &"getBoundingClientRect".into(),
        method.as_ref(),
    )
    .unwrap();
    setup.dom.element.dispatch_event(&wheel).unwrap();
    assert_eq!(
        order.borrow().as_slice(),
        &["client-get", "left-get", "client-coerce"]
    );
    js_sys::Reflect::delete_property(
        setup.dom.screen.unchecked_ref(),
        &"getBoundingClientRect".into(),
    )
    .unwrap();
}
