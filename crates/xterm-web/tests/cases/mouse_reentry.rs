#![cfg(target_arch = "wasm32")]
use super::support;

#[wasm_bindgen_test]
fn protocol_callback_failure_commits_mode_then_stops_dom_updates() {
    let setup = super::Setup::new();
    setup.host.fail_snapshot.set(true);
    assert!(setup.host.input.mouse.set_active_protocol("ANY").is_err());
    assert_eq!(setup.host.input.mouse.active_protocol(), "ANY");
    assert!(!setup
        .dom
        .element
        .class_list()
        .contains("enable-mouse-events"));
    let error = setup
        .mouse
        .take_protocol_error()
        .expect("preserve original JS getter exception");
    assert_eq!(
        js_sys::Reflect::get(&error, &"message".into())
            .unwrap()
            .as_string()
            .unwrap(),
        "snapshot getter failed"
    );
    setup.host.fail_snapshot.set(false);
    setup.host.input.mouse.set_active_protocol("ANY").unwrap();
    assert!(setup
        .dom
        .element
        .class_list()
        .contains("enable-mouse-events"));
    assert!(setup.mouse.take_protocol_error().is_none());
}

use std::{cell::Cell, rc::Rc};
use support::{input::event, Host};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;
use xterm_web::{
    dom::{BrowserDom, DomOptions},
    input::dom::mouse::MouseBindings,
};

#[wasm_bindgen_test]
fn wheel_custom_callback_can_reenter_the_same_dom_listener() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent = document
        .create_element("div")
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    let dom = BrowserDom::new(&parent, &DomOptions::default()).unwrap();
    let host = Rc::new(Host::new());
    let mut bindings = MouseBindings::new(&dom, host.clone()).unwrap();
    host.input.mouse.set_active_protocol("ANY").unwrap();
    host.input.mouse.set_active_encoding("SGR").unwrap();
    let rect = dom.screen.get_bounding_client_rect();
    let wheel = event(
        "WheelEvent",
        "wheel",
        &[
            ("clientX", (rect.left() + 4.0).into()),
            ("clientY", (rect.top() + 4.0).into()),
            ("deltaY", 1.into()),
            ("deltaMode", 1.into()),
        ],
    );
    let element = dom.element.clone();
    let guard = Rc::new(Cell::new(false));
    let inner = event(
        "WheelEvent",
        "wheel",
        &[
            ("clientX", (rect.left() + 4.0).into()),
            ("clientY", (rect.top() + 4.0).into()),
            ("deltaY", 1.into()),
            ("deltaMode", 1.into()),
        ],
    );
    *host.wheel_hook.borrow_mut() = Some(Rc::new(move |_| {
        if !guard.replace(true) {
            element.dispatch_event(&inner).unwrap();
        }
    }));
    dom.element.dispatch_event(&wheel).unwrap();
    assert_eq!(host.wheel_calls.get(), 2);
    assert_eq!(host.input.data.borrow().len(), 2);
    assert!(host.input.errors.borrow().is_empty());
    bindings.dispose().unwrap();
    parent.remove();
}

#[wasm_bindgen_test(async)]
async fn global_release_uses_elements_current_owner_document() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent = document
        .create_element("div")
        .unwrap()
        .dyn_into::<web_sys::HtmlElement>()
        .unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    let dom = BrowserDom::new(&parent, &DomOptions::default()).unwrap();
    let host = Rc::new(Host::new());
    let mut bindings = MouseBindings::new(&dom, host.clone()).unwrap();
    host.input.mouse.set_active_protocol("ANY").unwrap();
    host.input.mouse.set_active_encoding("SGR").unwrap();
    let frame = document.create_element("iframe").unwrap();
    document.body().unwrap().append_child(&frame).unwrap();
    support::input::next_turn().await;
    let next_document = js_sys::Reflect::get(&frame, &"contentDocument".into())
        .unwrap()
        .dyn_into::<web_sys::Document>()
        .unwrap();
    next_document
        .body()
        .unwrap()
        .append_child(&dom.element)
        .unwrap();
    let rect = dom.screen.get_bounding_client_rect();
    let fields = [
        ("clientX", (rect.left() + 4.0).into()),
        ("clientY", (rect.top() + 4.0).into()),
        ("button", 0.into()),
        ("buttons", 1.into()),
    ];
    dom.element
        .dispatch_event(&event("MouseEvent", "mousedown", &fields))
        .unwrap();
    assert_eq!(host.input.data.borrow().len(), 1);
    let release_fields = [
        ("clientX", (rect.left() + 4.0).into()),
        ("clientY", (rect.top() + 4.0).into()),
        ("button", 0.into()),
        ("buttons", 0.into()),
    ];
    document
        .dispatch_event(&event("MouseEvent", "mouseup", &release_fields))
        .unwrap();
    assert_eq!(
        host.input.data.borrow().len(),
        1,
        "old document has no global release listener"
    );
    next_document
        .dispatch_event(&event("MouseEvent", "mouseup", &release_fields))
        .unwrap();
    assert_eq!(host.input.data.borrow().len(), 2);
    bindings.dispose().unwrap();
    assert!(host.input.errors.borrow().is_empty());
    frame.remove();
    parent.remove();
}
