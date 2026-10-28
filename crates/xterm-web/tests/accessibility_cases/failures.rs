// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use crate::accessibility_support::{text, units, Fixture};
use js_sys::{Function, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::accessibility::AccessibilityEvent as Event;

#[wasm_bindgen_test]
fn throwing_label_coercion_preserves_original_value_and_partial_announcement_state() {
    let f = Fixture::new();
    let marker = Object::new();
    let label = Function::new_with_args("marker", "return {toString(){throw marker;}}")
        .call1(&JsValue::UNDEFINED, &marker)
        .unwrap();
    f.labels.set_too_much_output_value(label);
    for _ in 0..20 {
        f.host.emit(Event::LineFeed).unwrap();
    }
    assert_eq!(
        f.host.emit(Event::LineFeed).unwrap_err(),
        JsValue::from(marker.clone())
    );
    assert!(text(&f.live()).is_empty());
    f.labels.set_too_much_output(units("new label"));
    f.render();
    assert_eq!(text(&f.live()), vec![10; 21]);
    f.host
        .emit(Event::Character(units("still dropped")))
        .unwrap();
    f.render();
    assert_eq!(text(&f.live()), vec![10; 21]);
}
#[wasm_bindgen_test]
fn coercion_errors_keep_falsy_and_nan_thrown_values_without_generic_wrapping() {
    let f = Fixture::new();
    for marker in [
        JsValue::UNDEFINED,
        JsValue::NULL,
        JsValue::FALSE,
        JsValue::from_f64(0.0),
        JsValue::from_str(""),
        JsValue::from_f64(f64::NAN),
    ] {
        f.host.emit(Event::Blur).unwrap();
        let label = Function::new_with_args("marker", "return {toString(){throw marker;}}")
            .call1(&JsValue::UNDEFINED, &marker)
            .unwrap();
        f.labels.set_too_much_output_value(label);
        for _ in 0..20 {
            f.host.emit(Event::LineFeed).unwrap();
        }
        let actual = f.host.emit(Event::LineFeed).unwrap_err();
        assert!(Object::is(&actual, &marker));
    }
}
#[wasm_bindgen_test]
fn symbol_label_throws_at_dom_use_not_assignment_and_recovers_with_empty_label() {
    let f = Fixture::new();
    f.labels
        .set_too_much_output_value(js_sys::Symbol::for_("a11y-symbol").into());
    for _ in 0..20 {
        f.host.emit(Event::LineFeed).unwrap();
    }
    let error = f.host.emit(Event::LineFeed).unwrap_err();
    assert!(error.is_instance_of::<js_sys::TypeError>());
    f.labels.set_too_much_output(Vec::new());
    f.render();
    assert_eq!(text(&f.live()), vec![10; 21]);
    f.host.emit(Event::Character(vec![81])).unwrap();
    f.render();
    let mut expected = vec![10; 21];
    expected.push(81);
    assert_eq!(text(&f.live()), expected);
}
#[wasm_bindgen_test]
fn throwing_live_setter_retains_pending_output_and_source_timer_flags() {
    let f = Fixture::new();
    let marker = Object::new();
    let live = f.live();
    Function::new_with_args("element,marker", "Object.defineProperty(element,'textContent',{configurable:true,get(){return '';},set(){throw marker;}})")
        .call2(&JsValue::UNDEFINED, &live, &marker).unwrap();
    f.host.emit(Event::Character(vec![65])).unwrap();
    f.host.render().unwrap();
    assert_eq!(
        f.clock.advance(1000.0).unwrap_err(),
        JsValue::from(marker.clone())
    );
    Reflect::delete_property(&live, &"textContent".into()).unwrap();
    f.clock.advance(1000.0).unwrap();
    f.host.render().unwrap();
    assert_eq!(text(&live), [65]);
    assert_eq!(f.clock.pending(), 0);
}
#[wasm_bindgen_test]
fn reentrant_live_getter_can_append_without_overlapping_a_rust_borrow() {
    let f = Fixture::new();
    let live = f.live();
    let owner = Rc::new(RefCell::new(Some(f.manager.clone())));
    let weak = Rc::downgrade(&owner);
    let callback = Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || {
        let owner = weak
            .upgrade()
            .ok_or_else(|| js_sys::Error::new("test owner disappeared"))?;
        let manager = owner
            .borrow()
            .clone()
            .ok_or_else(|| js_sys::Error::new("test manager disappeared"))?;
        manager.handle_event(Event::Character(vec![66]))
    });
    Function::new_with_args("element,callback", "let value='';let first=true;Object.defineProperty(element,'textContent',{configurable:true,get(){if(first){first=false;callback();}return value;},set(v){value=v;}})")
        .call2(&JsValue::UNDEFINED, &live, callback.as_ref()).unwrap();
    f.host.emit(Event::Character(vec![65])).unwrap();
    f.render();
    assert_eq!(text(&live), [65, 66]);
    Reflect::delete_property(&live, &"textContent".into()).unwrap();
    owner.borrow_mut().take();
}
