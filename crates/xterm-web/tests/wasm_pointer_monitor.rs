#![cfg(target_arch = "wasm32")]
// Copyright (c) Microsoft Corporation. All rights reserved. MIT license.
//! Source-derived pointer-monitor assertions. Synthetic capture denial tests
//! complement, but do not establish, trusted pointer capture during a real drag.
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Element, EventTarget, HtmlIFrameElement, PointerEvent, PointerEventInit};
use xterm_web::scroll::pointer_monitor::GlobalPointerMoveMonitor;
wasm_bindgen_test_configure!(run_in_browser);
fn element() -> Element {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap()
}
fn target() -> EventTarget {
    web_sys::window().unwrap().into()
}
fn movement(buttons: u16) -> PointerEvent {
    let init = PointerEventInit::new();
    init.set_buttons(buttons);
    init.set_cancelable(true);
    PointerEvent::new_with_event_init_dict("pointermove", &init).unwrap()
}
fn up() -> PointerEvent {
    PointerEvent::new("pointerup").unwrap()
}
#[wasm_bindgen_test]
fn capture_denial_uses_window_prevents_valid_moves_and_stops_on_buttons_change() {
    let monitor = GlobalPointerMoveMonitor::new();
    let moved = Rc::new(Cell::new(0));
    let stopped = Rc::new(Cell::new(0));
    let count = moved.clone();
    let stop = stopped.clone();
    monitor
        .start_monitoring(
            element(),
            999.into(),
            1.into(),
            Rc::new(move |event| {
                assert!(event.default_prevented());
                count.set(count.get() + 1);
                Ok(())
            }),
            Rc::new(move || {
                stop.set(stop.get() + 1);
                Ok(())
            }),
        )
        .unwrap();
    let event = movement(1);
    assert!(!target().dispatch_event(event.as_ref()).unwrap());
    assert_eq!(moved.get(), 1);
    let event = movement(0);
    assert!(target().dispatch_event(event.as_ref()).unwrap());
    assert_eq!(stopped.get(), 1);
    assert!(!monitor.is_monitoring());
    target().dispatch_event(up().as_ref()).unwrap();
    assert_eq!(stopped.get(), 1);
    monitor.dispose();
}
#[wasm_bindgen_test]
fn replacing_monitor_does_not_call_previous_stop_callback() {
    let monitor = GlobalPointerMoveMonitor::new();
    let old = Rc::new(Cell::new(0));
    let stopped = Rc::new(Cell::new(0));
    let count = old.clone();
    let stop = stopped.clone();
    monitor
        .start_monitoring(
            element(),
            999.into(),
            1.into(),
            Rc::new(move |_| {
                count.set(count.get() + 1);
                Ok(())
            }),
            Rc::new(move || {
                stop.set(stop.get() + 1);
                Ok(())
            }),
        )
        .unwrap();
    let new = Rc::new(Cell::new(0));
    let count = new.clone();
    monitor
        .start_monitoring(
            element(),
            998.into(),
            2.into(),
            Rc::new(move |_| {
                count.set(count.get() + 1);
                Ok(())
            }),
            Rc::new(|| Ok(())),
        )
        .unwrap();
    target().dispatch_event(movement(2).as_ref()).unwrap();
    assert_eq!(new.get(), 1);
    assert_eq!(old.get(), 0);
    assert_eq!(stopped.get(), 0);
    monitor.dispose();
}
#[wasm_bindgen_test]
fn stop_callback_can_start_another_monitor_without_losing_its_hooks() {
    let monitor = GlobalPointerMoveMonitor::new();
    let weak = monitor.downgrade();
    let moved = Rc::new(Cell::new(0));
    let count = moved.clone();
    monitor
        .start_monitoring(
            element(),
            999.into(),
            1.into(),
            Rc::new(|_| Ok(())),
            Rc::new(move || {
                let count = count.clone();
                weak.upgrade().unwrap().start_monitoring(
                    element(),
                    998.into(),
                    2.into(),
                    Rc::new(move |_| {
                        count.set(count.get() + 1);
                        Ok(())
                    }),
                    Rc::new(|| Ok(())),
                )
            }),
        )
        .unwrap();
    target().dispatch_event(up().as_ref()).unwrap();
    assert!(monitor.is_monitoring());
    target().dispatch_event(movement(2).as_ref()).unwrap();
    assert_eq!(moved.get(), 1);
    monitor.dispose();
}
#[wasm_bindgen_test]
fn null_late_target_and_raw_button_identity_follow_source() {
    let monitor = GlobalPointerMoveMonitor::new();
    let moved = Rc::new(Cell::new(0));
    let count = moved.clone();
    // A source caller can reread a target that became null. Its capture error
    // still falls back to window, rather than fabricating a target/id.
    monitor
        .start_monitoring_raw(
            JsValue::NULL,
            JsValue::UNDEFINED,
            1.into(),
            Rc::new(move |_| {
                count.set(count.get() + 1);
                Ok(())
            }),
            Rc::new(|| Ok(())),
        )
        .unwrap();
    target().dispatch_event(movement(1).as_ref()).unwrap();
    assert_eq!(moved.get(), 1);
    monitor.dispose();
}
#[wasm_bindgen_test]
fn raw_buttons_use_strict_identity_without_numeric_coercion() {
    let monitor = GlobalPointerMoveMonitor::new();
    let moved = Rc::new(Cell::new(0));
    let count = moved.clone();
    monitor
        .start_monitoring_raw(
            JsValue::NULL,
            JsValue::UNDEFINED,
            "1".into(),
            Rc::new(move |_| {
                count.set(count.get() + 1);
                Ok(())
            }),
            Rc::new(|| Ok(())),
        )
        .unwrap();
    let event = web_sys::Event::new("pointermove").unwrap();
    assert!(js_sys::Reflect::set(event.as_ref(), &"buttons".into(), &"1".into()).unwrap());
    target().dispatch_event(&event).unwrap();
    assert_eq!(moved.get(), 1);
    target().dispatch_event(movement(1).as_ref()).unwrap();
    assert!(!monitor.is_monitoring());
    assert_eq!(moved.get(), 1);
    monitor.dispose();
}

#[wasm_bindgen_test]
fn owner_iframe_window_is_used_instead_of_the_module_window() {
    let document = web_sys::window().unwrap().document().unwrap();
    let iframe = document
        .create_element("iframe")
        .unwrap()
        .dyn_into::<HtmlIFrameElement>()
        .unwrap();
    document.body().unwrap().append_child(&iframe).unwrap();
    let foreign = iframe
        .content_document()
        .unwrap()
        .create_element("div")
        .unwrap();
    let window: EventTarget = iframe.content_window().unwrap().into();
    let monitor = GlobalPointerMoveMonitor::new();
    let moved = Rc::new(Cell::new(0));
    let count = moved.clone();
    monitor
        .start_monitoring(
            foreign,
            999.into(),
            1.into(),
            Rc::new(move |_| {
                count.set(count.get() + 1);
                Ok(())
            }),
            Rc::new(|| Ok(())),
        )
        .unwrap();
    target().dispatch_event(movement(1).as_ref()).unwrap();
    assert_eq!(moved.get(), 0);
    window.dispatch_event(movement(1).as_ref()).unwrap();
    assert_eq!(moved.get(), 1);
    monitor.dispose();
    iframe.remove();
}
#[wasm_bindgen_test]
fn dispose_and_drop_release_hooks_without_invoking_stop() {
    let monitor = GlobalPointerMoveMonitor::new();
    let weak = monitor.downgrade();
    let stopped = Rc::new(Cell::new(0));
    let count = stopped.clone();
    monitor
        .start_monitoring(
            element(),
            999.into(),
            1.into(),
            Rc::new(|_| Ok(())),
            Rc::new(move || {
                count.set(count.get() + 1);
                Ok(())
            }),
        )
        .unwrap();
    monitor.dispose();
    assert!(!monitor.is_monitoring());
    target().dispatch_event(up().as_ref()).unwrap();
    assert_eq!(stopped.get(), 0);
    drop(monitor);
    assert!(weak.upgrade().is_none());
}
#[wasm_bindgen_test]
fn after_dispose_start_keeps_the_source_closed_store_semantics() {
    let monitor = GlobalPointerMoveMonitor::new();
    monitor.dispose();
    let moved = Rc::new(Cell::new(0));
    let stopped = Rc::new(Cell::new(0));
    let count = moved.clone();
    let stop = stopped.clone();
    monitor
        .start_monitoring(
            element(),
            999.into(),
            1.into(),
            Rc::new(move |_| {
                count.set(count.get() + 1);
                Ok(())
            }),
            Rc::new(move || {
                stop.set(stop.get() + 1);
                Ok(())
            }),
        )
        .unwrap();
    assert!(monitor.is_monitoring());
    target().dispatch_event(movement(1).as_ref()).unwrap();
    assert_eq!(moved.get(), 0);
    monitor.stop_monitoring(true).unwrap();
    assert_eq!(stopped.get(), 1);
    assert!(!monitor.is_monitoring());
}
