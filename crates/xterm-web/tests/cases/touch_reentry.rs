use super::support::{number, send, Setup};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_core::lifecycle::DisposableHandle;
use xterm_web::dom_listener::DomListener;

#[wasm_bindgen_test]
fn containment_callbacks_observe_live_target_and_ignore_list_insertions() {
    let mut setup = Setup::new();
    setup.track(true);
    setup.track(false);
    let token = setup.context.add_target(&setup.child).unwrap();
    let added = Rc::new(RefCell::new(None::<DisposableHandle>));
    let captured = Rc::clone(&added);
    let weak = Rc::downgrade(&setup.context);
    let child = setup.child.clone();
    let parent = setup.parent.clone();
    let original = js_sys::Reflect::get(&child, &"contains".into()).unwrap();
    let callback_original: js_sys::Function = original.clone().dyn_into().unwrap();
    let callback = Closure::wrap(Box::new(move |node: JsValue| -> Result<JsValue, JsValue> {
        if captured.borrow().is_none() {
            let token = weak.upgrade().unwrap().add_target(&parent)?;
            *captured.borrow_mut() = Some(token);
        }
        callback_original.call1(&child, &node)
    }) as Box<dyn Fn(JsValue) -> Result<JsValue, JsValue>>);
    js_sys::Reflect::set(&setup.child, &"contains".into(), callback.as_ref()).unwrap();
    setup.send("touchstart", 1, 0.0, 0.0);
    assert_eq!(
        setup
            .log
            .borrow()
            .iter()
            .map(|entry| entry.0.as_str())
            .collect::<Vec<_>>(),
        ["child", "parent"]
    );
    js_sys::Reflect::set(&setup.child, &"contains".into(), &original).unwrap();
    drop(callback);

    let detached: web_sys::HtmlElement = setup
        .parent
        .owner_document()
        .unwrap()
        .create_element("div")
        .unwrap()
        .dyn_into()
        .unwrap();
    let ignored = setup.context.ignore_target(&detached).unwrap();
    let added_ignore = Rc::new(RefCell::new(None::<DisposableHandle>));
    let captured = Rc::clone(&added_ignore);
    let weak = Rc::downgrade(&setup.context);
    let child = setup.child.clone();
    let callback = Closure::wrap(Box::new(move |_: JsValue| -> Result<bool, JsValue> {
        if captured.borrow().is_none() {
            let token = weak.upgrade().unwrap().ignore_target(&child)?;
            *captured.borrow_mut() = Some(token);
        }
        Ok(false)
    }) as Box<dyn Fn(JsValue) -> Result<bool, JsValue>>);
    js_sys::Reflect::set(&detached, &"contains".into(), callback.as_ref()).unwrap();
    setup.clear();
    assert!(!setup.send("touchstart", 2, 0.0, 0.0).default_prevented());
    assert!(setup.log.borrow().is_empty());
    js_sys::Reflect::delete_property(&detached, &"contains".into()).unwrap();
    drop(callback);
    ignored.dispose();
    added_ignore.borrow().as_ref().unwrap().dispose();
    added.borrow().as_ref().unwrap().dispose();
    token.dispose();
}

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
