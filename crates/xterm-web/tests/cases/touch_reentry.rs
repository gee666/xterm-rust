use super::support::{number, send, Setup};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::closure::Closure;
use wasm_bindgen_test::*;
use xterm_web::dom_listener::DomListener;

#[wasm_bindgen_test]
fn cancellation_flag_clears_after_both_event_cancellation_methods() {
    let mut setup = Setup::new();
    setup.track(true);
    let token = setup.context.add_target(&setup.child).unwrap();
    let body = setup.parent.owner_document().unwrap().body().unwrap();
    let observed = Rc::new(Cell::new(false));
    let captured = Rc::clone(&observed);
    let callback = Closure::wrap(Box::new(move || {
        // The new touch is outside all registered targets. Source still sees
        // the outer dispatched flag until its cancellation methods return.
        let nested = send(&body, "touchstart", 9, 0.0, 0.0);
        captured.set(nested.default_prevented());
    }) as Box<dyn Fn()>);
    let function = callback.as_ref().clone();
    let listener = DomListener::new(setup.leaf.as_ref(), "touchstart", false, move |event| {
        js_sys::Reflect::set(&event, &"preventDefault".into(), &function).unwrap();
    })
    .unwrap();
    setup.send("touchstart", 1, 0.0, 0.0);
    assert!(observed.get());
    drop(listener);
    drop(callback);
    token.dispose();
}

#[wasm_bindgen_test]
fn inertial_changes_do_not_reset_tap_timestamp_from_nested_touch() {
    let mut setup = Setup::new();
    setup.track(true);
    let token = setup.context.add_target(&setup.child).unwrap();
    setup.send("touchstart", 1, 0.0, 0.0);
    setup.time(1010.0);
    setup.send("touchmove", 1, 100.0, 0.0);
    setup.time(1020.0);
    setup.send("touchend", 1, 100.0, 0.0);
    let entered = Rc::new(Cell::new(false));
    let captured = Rc::clone(&entered);
    let leaf = setup.leaf.clone();
    let listener = DomListener::new(
        setup.child.as_ref(),
        "-xterm-gesturechange",
        false,
        move |_| {
            if !captured.replace(true) {
                send(&leaf, "touchstart", 2, 0.0, 0.0);
                send(&leaf, "touchend", 2, 0.0, 0.0);
            }
        },
    )
    .unwrap();
    setup.time(1030.0);
    setup.flush();
    assert!(entered.get());
    setup.time(1040.0);
    setup.flush();
    setup.clear();
    setup.send("touchstart", 3, 0.0, 0.0);
    setup.send("touchend", 3, 0.0, 0.0);
    assert_eq!(number(&setup.log.borrow()[1].1, "tapCount"), 2.0);
    drop(listener);
    token.dispose();
}

#[wasm_bindgen_test]
fn reentrant_start_replaces_touch_record_before_outer_move_commits() {
    let mut setup = Setup::new();
    setup.track(true);
    let target = setup.context.add_target(&setup.child).unwrap();
    let entered = Rc::new(Cell::new(false));
    let captured = Rc::clone(&entered);
    let leaf = setup.leaf.clone();
    let listener = DomListener::new(
        setup.child.as_ref(),
        "-xterm-gesturechange",
        false,
        move |_| {
            if !captured.replace(true) {
                send(&leaf, "touchstart", 1, 100.0, 0.0);
            }
        },
    )
    .unwrap();
    setup.send("touchstart", 1, 0.0, 0.0);
    setup.time(1010.0);
    setup.send("touchmove", 1, 50.0, 0.0);
    setup.clear();
    setup.time(1020.0);
    setup.send("touchmove", 1, 110.0, 0.0);
    assert_eq!(number(&setup.log.borrow()[0].1, "translationX"), 10.0);
    drop(listener);
    target.dispose();
}

#[wasm_bindgen_test]
fn reentrant_end_replacement_is_deleted_after_end_dispatch() {
    let mut setup = Setup::new();
    setup.track(true);
    let target = setup.context.add_target(&setup.child).unwrap();
    let leaf = setup.leaf.clone();
    let listener = DomListener::new(
        setup.child.as_ref(),
        "-xterm-gesturesend",
        false,
        move |_| {
            send(&leaf, "touchstart", 1, 100.0, 0.0);
        },
    )
    .unwrap();
    setup.send("touchstart", 1, 0.0, 0.0);
    setup.send("touchend", 1, 0.0, 0.0);
    setup.clear();
    setup.send("touchmove", 1, 110.0, 0.0);
    assert!(setup.log.borrow().is_empty());
    drop(listener);
    target.dispose();
}
