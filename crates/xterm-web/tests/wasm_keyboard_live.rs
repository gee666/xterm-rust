#![cfg(target_arch = "wasm32")]
#[path = "support/live_key_dom.rs"]
mod live_support;
#[path = "support/input_dom.rs"]
mod support;
use live_support::*;
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use support::*;
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_test::*;
use web_sys::Event;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn custom_veto_does_not_read_any_event_field() {
    let setup = Setup::new();
    let errors = Errors::new();
    setup.host.veto.set(true);
    let event = down("a", "KeyA", 65);
    let reads = Rc::new(Cell::new(0));
    let observed = reads.clone();
    let marker: JsValue = js_sys::Object::new().into();
    let getter = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(move || {
        observed.set(observed.get() + 1);
        Err(marker.clone())
    });
    for field in [
        "key",
        "code",
        "shiftKey",
        "altKey",
        "ctrlKey",
        "metaKey",
        "keyCode",
        "charCode",
        "which",
        "repeat",
        "type",
        "getModifierState",
    ] {
        define(&event, field, "get", getter.as_ref());
    }
    setup.dom.textarea.dispatch_event(&event).unwrap();
    assert_eq!(reads.get(), 0);
    assert!(setup.host.data.borrow().is_empty());
    assert!(setup.host.effects.borrow().is_empty());
    errors.empty();
}

#[wasm_bindgen_test]
fn custom_handler_replaces_fields_before_composition_and_encoding() {
    let setup = Setup::new();
    let errors = Errors::new();
    *setup.host.custom_key.borrow_mut() = Some(Rc::new(|event| {
        let event: &Event = event.as_ref();
        for (name, value) in [
            ("key", "ArrowUp".into()),
            ("code", "ArrowUp".into()),
            ("keyCode", 38.into()),
        ] {
            define(event, name, "value", &value);
        }
        Ok(true)
    }));
    let event = down("Enter", "Enter", 13);
    setup.dom.textarea.dispatch_event(&event).unwrap();
    assert_eq!(*setup.host.data.borrow(), vec![(units("\x1b[A"), true)]);
    assert!(event.default_prevented());
    errors.empty();
}

#[wasm_bindgen_test]
fn unused_legacy_code_getter_does_not_throw() {
    let setup = Setup::new();
    let errors = Errors::new();
    let event = down("Tab", "Tab", 9);
    let getter =
        Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(|| Err("unused code getter".into()));
    define(&event, "code", "get", getter.as_ref());
    setup.dom.textarea.dispatch_event(&event).unwrap();
    assert_eq!(*setup.host.data.borrow(), vec![(vec![9], true)]);
    errors.empty();
}

#[wasm_bindgen_test]
fn thrown_getter_retains_original_js_value_and_stops_before_cursor_effect() {
    let setup = Setup::new();
    let errors = Errors::new();
    let marker: JsValue = js_sys::Object::new().into();
    let thrown = marker.clone();
    let event = down("a", "KeyA", 65);
    let getter = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(move || Err(thrown.clone()));
    define(&event, "key", "get", getter.as_ref());
    setup.dom.textarea.dispatch_event(&event).unwrap();
    assert_eq!(*errors.values.borrow(), vec![marker]);
    assert!(setup.host.data.borrow().is_empty());
    assert!(setup.host.effects.borrow().is_empty());
}

#[wasm_bindgen_test]
fn third_level_checks_type_even_when_no_platform_modifier_matches() {
    let setup = Setup::new();
    let errors = Errors::new();
    let marker: JsValue = js_sys::Object::new().into();
    let thrown = marker.clone();
    let event = down("Enter", "Enter", 13);
    let getter = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(move || Err(thrown.clone()));
    define(&event, "type", "get", getter.as_ref());
    setup.dom.textarea.dispatch_event(&event).unwrap();
    assert_eq!(*errors.values.borrow(), vec![marker]);
    assert_eq!(
        *setup.host.effects.borrow(),
        vec!["UpdateCursorStyle".to_owned()]
    );
    assert!(setup.host.data.borrow().is_empty());
    assert!(!event.default_prevented());
}

#[wasm_bindgen_test]
fn onkey_mutates_final_screen_reader_cancellation_without_changing_encoded_key() {
    let setup = Setup::new();
    let errors = Errors::new();
    let mut snapshot = setup.host.keyboard.get();
    snapshot.screen_reader_mode = true;
    setup.host.keyboard.set(snapshot);
    let event = down("ArrowUp", "ArrowUp", 38);
    let mutated = event.clone();
    *setup.host.on_key.borrow_mut() = Some(Rc::new(move || {
        define(&mutated, "altKey", "value", &true.into())
    }));
    setup.dom.textarea.dispatch_event(&event).unwrap();
    assert_eq!(*setup.host.data.borrow(), vec![(units("\x1b[A"), true)]);
    assert!(
        event.default_prevented(),
        "final Alt read must observe onKey mutation"
    );
    errors.empty();
}

#[wasm_bindgen_test]
fn keypress_repeats_which_reads_and_distinguishes_undefined_from_null() {
    let setup = Setup::new();
    let errors = Errors::new();
    let reads = Rc::new(RefCell::new(Vec::new()));
    let chars = Rc::new(Cell::new(0));
    let count = chars.clone();
    let out = reads.clone();
    let char_getter = Closure::<dyn Fn() -> JsValue>::new(move || {
        out.borrow_mut().push("char");
        let previous = count.replace(count.get() + 1);
        if previous == 0 {
            0.into()
        } else {
            65.into()
        }
    });
    let which = Rc::new(Cell::new(0));
    let count = which.clone();
    let out = reads.clone();
    let which_getter = Closure::<dyn Fn() -> JsValue>::new(move || {
        out.borrow_mut().push("which");
        let previous = count.replace(count.get() + 1);
        if previous == 0 {
            JsValue::UNDEFINED
        } else {
            97.into()
        }
    });
    let event = event("KeyboardEvent", "keypress", &[("keyCode", 0.into())]);
    define(&event, "charCode", "get", char_getter.as_ref());
    define(&event, "which", "get", which_getter.as_ref());
    setup.dom.textarea.dispatch_event(&event).unwrap();
    assert_eq!(
        *reads.borrow(),
        ["char", "which", "which", "which", "char", "which"]
    );
    assert_eq!(*setup.host.data.borrow(), vec![(vec![97], true)]);
    errors.empty();
}

#[wasm_bindgen_test]
fn kitty_does_not_eagerly_reread_keycode_for_encoding() {
    let setup = Setup::new();
    let errors = Errors::new();
    let mut snapshot = setup.host.keyboard.get();
    snapshot.modes.kitty_extension = true;
    snapshot.modes.kitty_flags = 31;
    setup.host.keyboard.set(snapshot);
    let event = down("a", "KeyA", 65);
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let getter = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(move || {
        let previous = count.replace(count.get() + 1);
        if previous == 0 {
            Ok(65.into())
        } else {
            Err("unused Kitty keyCode".into())
        }
    });
    define(&event, "keyCode", "get", getter.as_ref());
    setup.dom.textarea.dispatch_event(&event).unwrap();
    assert_eq!(calls.get(), 1);
    assert_eq!(
        *setup.host.data.borrow(),
        vec![(units("\x1b[97;;97u"), true)]
    );
    errors.empty();
}

#[wasm_bindgen_test(async)]
async fn active_ime_reads_keycode_again_and_retains_composition_until_end() {
    let setup = Setup::new();
    let errors = Errors::new();
    setup
        .dom
        .textarea
        .dispatch_event(&event("CompositionEvent", "compositionstart", &[]))
        .unwrap();
    let key = down("a", "KeyA", 230);
    let count = Rc::new(Cell::new(0));
    let reads = count.clone();
    let getter = Closure::<dyn Fn() -> JsValue>::new(move || {
        let previous = reads.replace(reads.get() + 1);
        if previous == 0 {
            230.into()
        } else {
            16.into()
        }
    });
    define(&key, "keyCode", "get", getter.as_ref());
    let forbidden = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(|| {
        Err("IME rejected key must not reach encoder".into())
    });
    define(&key, "key", "get", forbidden.as_ref());
    setup.dom.textarea.dispatch_event(&key).unwrap();
    assert_eq!(count.get(), 2);
    assert!(setup.host.data.borrow().is_empty());
    setup.dom.textarea.set_value("中");
    setup
        .dom
        .textarea
        .dispatch_event(&event(
            "CompositionEvent",
            "compositionend",
            &[("data", "中".into())],
        ))
        .unwrap();
    next_turn().await;
    assert_eq!(*setup.host.data.borrow(), vec![(units("中"), true)]);
    errors.empty();
}

#[wasm_bindgen_test]
fn custom_handler_reenters_then_mutates_outer_event_without_held_borrows() {
    let setup = Setup::new();
    let errors = Errors::new();
    let nested = down("ArrowUp", "ArrowUp", 38);
    let area = setup.dom.textarea.clone();
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    *setup.host.custom_key.borrow_mut() = Some(Rc::new(move |outer| {
        let previous = count.replace(count.get() + 1);
        if previous == 0 {
            area.dispatch_event(&nested)?;
            let outer: &Event = outer.as_ref();
            define(outer, "key", "value", &"ArrowDown".into());
            define(outer, "code", "value", &"ArrowDown".into());
            define(outer, "keyCode", "value", &40.into());
        }
        Ok(true)
    }));
    setup
        .dom
        .textarea
        .dispatch_event(&down("Enter", "Enter", 13))
        .unwrap();
    assert_eq!(calls.get(), 2);
    assert_eq!(
        *setup.host.data.borrow(),
        vec![(units("\x1b[A"), true), (units("\x1b[B"), true)]
    );
    errors.empty();
}

#[wasm_bindgen_test]
fn win32_reads_code_at_each_virtual_scan_modifier_and_enhanced_stage() {
    let setup = Setup::new();
    let errors = Errors::new();
    let mut snapshot = setup.host.keyboard.get();
    snapshot.modes.win32_extension = true;
    snapshot.modes.win32_mode = true;
    setup.host.keyboard.set(snapshot);
    let event = down("a", "KeyA", 65);
    define(&event, "ctrlKey", "value", &true.into());
    define(&event, "altKey", "value", &true.into());
    let calls = Rc::new(Cell::new(0));
    let count = calls.clone();
    let getter = Closure::<dyn Fn() -> JsValue>::new(move || {
        let index = count.replace(count.get() + 1);
        ["KeyA", "KeyB", "ControlRight", "AltRight", "ArrowRight"][index.min(4)].into()
    });
    define(&event, "code", "get", getter.as_ref());
    setup.dom.textarea.dispatch_event(&event).unwrap();
    assert_eq!(calls.get(), 5);
    assert_eq!(
        *setup.host.data.borrow(),
        vec![(units("\x1b[65;48;97;1;261;1_"), true)]
    );
    errors.empty();
}
