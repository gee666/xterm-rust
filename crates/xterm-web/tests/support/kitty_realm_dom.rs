// Licensed under MIT. Real textarea listeners, dispatch and shared table owner.
use super::{live_support::*, support::*};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::utf16::from_js_string;

#[wasm_bindgen(module = "/tests/fixtures/kitty_lookup_driver.js")]
extern "C" {
    #[wasm_bindgen(js_name = kittySourceExpected)]
    fn expected(event: &JsValue) -> js_sys::JsString;
    #[wasm_bindgen(js_name = installKittyDomProbe)]
    fn install(callback: &js_sys::Function) -> js_sys::Function;
    #[wasm_bindgen(js_name = kittyThrowValue)]
    fn throwing_value(marker: &JsValue) -> JsValue;
}
struct PrototypeProbe(js_sys::Function);
impl Drop for PrototypeProbe {
    fn drop(&mut self) {
        self.0.call0(&JsValue::UNDEFINED).unwrap();
    }
}
fn kitty(setup: &Setup, enabled: bool) {
    let mut snapshot = setup.host.keyboard.get();
    snapshot.modes.kitty_extension = enabled;
    snapshot.modes.kitty_flags = 8;
    setup.host.keyboard.set(snapshot);
}

#[wasm_bindgen_test]
fn kitty_dom_uses_this_realms_intrinsic_function_text_after_custom_callback() {
    let setup = Setup::new();
    let errors = Errors::new();
    kitty(&setup, true);
    *setup.host.custom_key.borrow_mut() = Some(Rc::new(|event| {
        define(event.as_ref(), "key", "value", &"toString".into());
        Ok(true)
    }));
    let key = down("a", "KeyA", 65);
    let source = down("toString", "KeyA", 65);
    let output = from_js_string(&expected(source.as_ref()));
    setup.dom.textarea.dispatch_event(&key).unwrap();
    assert_eq!(*setup.host.data.borrow(), vec![(output, true)]);
    assert!(key.default_prevented());
    errors.empty();
}

#[wasm_bindgen_test]
fn kitty_dom_keeps_one_table_across_modes_reentry_and_disposes_listener() {
    let setup = Setup::new();
    let errors = Errors::new();
    kitty(&setup, true);
    let receivers = Rc::new(RefCell::new(Vec::new()));
    let output = receivers.clone();
    let area = setup.dom.textarea.clone();
    let getter = Closure::<dyn Fn(JsValue) -> JsValue>::new(move |receiver| {
        // Release the trace borrow before entering the live DOM listener again.
        let nested = output.borrow().is_empty();
        output.borrow_mut().push(receiver);
        if nested {
            area.dispatch_event(&down("a", "KeyA", 65)).unwrap();
        }
        "receiver-value".into()
    });
    let _probe = PrototypeProbe(install(getter.as_ref().unchecked_ref()));
    let key = down("__kitty_dom_probe__", "", 0);
    setup.dom.textarea.dispatch_event(&key).unwrap();
    assert_eq!(
        *setup.host.data.borrow(),
        vec![
            (units("\x1b[97u"), true),
            (units("\x1b[receiver-value"), true)
        ]
    );
    kitty(&setup, false);
    setup
        .dom
        .textarea
        .dispatch_event(&down("Enter", "Enter", 13))
        .unwrap();
    kitty(&setup, true);
    setup
        .dom
        .textarea
        .dispatch_event(&down("__kitty_dom_probe__", "", 0))
        .unwrap();
    let seen = receivers.borrow();
    assert_eq!(seen.len(), 2);
    assert_eq!(
        seen[0], seen[1],
        "mode changes must not recreate Kitty maps"
    );
    drop(seen);
    assert_eq!(setup.host.data.borrow().len(), 4);
    setup.input.dispose().unwrap();
    setup
        .dom
        .textarea
        .dispatch_event(&down("__kitty_dom_probe__", "", 0))
        .unwrap();
    assert_eq!(setup.host.data.borrow().len(), 4);
    assert_eq!(receivers.borrow().len(), 2);
    errors.empty();
}

#[wasm_bindgen_test]
fn kitty_dom_propagates_raw_conversion_error_before_onkey_and_data() {
    let setup = Setup::new();
    let errors = Errors::new();
    kitty(&setup, true);
    let marker: JsValue = js_sys::Object::new().into();
    let raw = throwing_value(&marker);
    let getter = Closure::<dyn Fn(JsValue) -> JsValue>::new(move |_| raw.clone());
    let _probe = PrototypeProbe(install(getter.as_ref().unchecked_ref()));
    let key = down("__kitty_dom_probe__", "", 0);
    setup.dom.textarea.dispatch_event(&key).unwrap();
    assert_eq!(*errors.values.borrow(), vec![marker]);
    assert!(setup.host.data.borrow().is_empty());
    assert!(setup.host.effects.borrow().is_empty());
    assert!(!key.default_prevented());
}
