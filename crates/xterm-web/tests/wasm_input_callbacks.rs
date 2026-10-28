#![cfg(target_arch = "wasm32")]
#[path = "support/input_dom.rs"]
mod support;
use std::{cell::Cell, rc::Rc};
use support::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn onkey_can_reenter_the_same_dom_listener_without_borrowing_it_mutably() {
    let setup = Setup::new();
    let nested = event(
        "KeyboardEvent",
        "keydown",
        &[
            ("keyCode", 38.into()),
            ("key", "ArrowUp".into()),
            ("code", "ArrowUp".into()),
        ],
    );
    let area = setup.dom.textarea.clone();
    let calls = Rc::new(Cell::new(0));
    let observed = calls.clone();
    *setup.host.on_key.borrow_mut() = Some(Rc::new(move || {
        let count = observed.get();
        observed.set(count + 1);
        if count == 0 {
            area.dispatch_event(&nested).unwrap();
        }
    }));
    let enter = event(
        "KeyboardEvent",
        "keydown",
        &[
            ("keyCode", 13.into()),
            ("key", "Enter".into()),
            ("code", "Enter".into()),
        ],
    );
    setup.dom.textarea.dispatch_event(&enter).unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(
        *setup.host.data.borrow(),
        vec![(units("\x1b[A"), true), (vec![13], true)]
    );
    assert!(enter.default_prevented());
}

#[wasm_bindgen_test]
fn paste_preserves_callback_error_identity_and_does_not_clear_after_throw() {
    let setup = Setup::new();
    let thrown: JsValue = js_sys::Object::new().into();
    *setup.host.data_error.borrow_mut() = Some(thrown.clone());
    setup.dom.textarea.set_value("keep after callback failure");
    assert_eq!(setup.input.paste(&units("new input")).unwrap_err(), thrown);
    assert_eq!(setup.dom.textarea.value(), "keep after callback failure");
    assert!(setup.host.data.borrow().is_empty());
}

#[wasm_bindgen_test(async)]
async fn standalone_229_samples_textarea_after_the_event_and_coalesces() {
    let setup = Setup::new();
    setup.dom.textarea.set_value("ab");
    let composition_key = event("KeyboardEvent", "keydown", &[("keyCode", 229.into())]);
    setup.dom.textarea.dispatch_event(&composition_key).unwrap();
    setup.dom.textarea.dispatch_event(&composition_key).unwrap();
    setup.dom.textarea.set_value("ab中");
    assert!(setup.host.data.borrow().is_empty());
    next_turn().await;
    assert_eq!(*setup.host.data.borrow(), vec![(units("中"), true)]);
}
