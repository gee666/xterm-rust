#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect, JSON};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::unicode::UnicodeCharProperties;
use xterm_web::Terminal;

#[wasm_bindgen(module = "/tests/support/unicode_api.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = unicodeTrace)]
    fn unicode_trace(api: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = registerMutable)]
    fn register_mutable(api: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = registerThrowing)]
    fn register_throwing(api: &JsValue, after_first: u32) -> Result<JsValue, JsValue>;
}
wasm_bindgen_test_configure!(run_in_browser);

fn get(value: &JsValue, key: &str) -> JsValue {
    Reflect::get(value, &key.into()).unwrap()
}
fn invoke(value: &JsValue, key: &str, argument: &JsValue) {
    get(value, key)
        .dyn_into::<Function>()
        .unwrap()
        .call1(value, argument)
        .unwrap();
}
fn terminal() -> Terminal {
    Terminal::new(JSON::parse(r#"{"allowProposedApi":true}"#).unwrap()).unwrap()
}

#[wasm_bindgen_test]
fn original_registry_trace_roundtrips_every_utf16_key_without_collisions() {
    let terminal = terminal();
    let api: JsValue = terminal.unicode().unwrap().into();
    let actual = unicode_trace(&api).unwrap();
    let expected = JSON::parse(include_str!("fixtures/unicode-api.json")).unwrap();
    assert_eq!(
        JSON::stringify(&actual).unwrap(),
        JSON::stringify(&expected).unwrap()
    );
    terminal.dispose().unwrap();
}

#[wasm_bindgen_test]
fn getter_is_fresh_gated_and_retained_views_stay_live_after_option_changes_and_disposal() {
    let terminal = terminal();
    let first = terminal.unicode().unwrap();
    let second: JsValue = terminal.unicode().unwrap().into();
    let first: JsValue = first.into();
    assert!(!js_sys::Object::is(&first, &second));
    let options: JsValue = terminal.options().into();
    Reflect::set(&options, &"allowProposedApi".into(), &JsValue::FALSE).unwrap();
    assert!(terminal.unicode().err().unwrap().is_object());
    assert!(Reflect::set(&first, &"activeVersion".into(), &"6".into()).unwrap());
    assert_eq!(get(&second, "activeVersion"), JsValue::from("6"));
    terminal.dispose().unwrap();
    assert_eq!(get(&first, "activeVersion"), JsValue::from("6"));
}

#[wasm_bindgen_test]
fn provider_callbacks_remain_live_keep_this_and_preserve_failure_identity() {
    let terminal = terminal();
    let service = terminal.shared_native().unicode().unwrap();
    let api: JsValue = terminal.unicode().unwrap().into();
    let host = register_mutable(&api).unwrap();
    assert_eq!(
        service
            .char_properties(65, UnicodeCharProperties::default())
            .unwrap()
            .width(),
        1
    );
    invoke(&host, "setWide", &JsValue::TRUE);
    assert_eq!(service.wcwidth(0xd800).unwrap().columns(), 2);
    assert_eq!(
        service
            .char_properties(66, UnicodeCharProperties::from_raw(2))
            .unwrap()
            .width(),
        2
    );
    let native = terminal.shared_native();
    native.write("A").unwrap();
    assert_eq!(
        native
            .buffer()
            .unwrap()
            .active()
            .unwrap()
            .cursor_x()
            .unwrap(),
        2
    );
    invoke(&host, "replace", &JsValue::UNDEFINED);
    assert_eq!(
        service.wcwidth(65).unwrap().columns(),
        2,
        "active snapshot is not replaced by registration"
    );
    invoke(&host, "setFail", &JsValue::TRUE);
    for error in [
        service.wcwidth(65).unwrap_err(),
        service
            .char_properties(65, UnicodeCharProperties::default())
            .unwrap_err(),
    ] {
        let value = xterm_web::js_error::to_js_error(&error);
        assert!(js_sys::Object::is(&value, &get(&host, "failure")));
    }
    invoke(&host, "setFail", &JsValue::FALSE);
    assert_eq!(
        service.wcwidth(65).unwrap().columns(),
        2,
        "failure releases borrows"
    );
    let trace = JSON::stringify(&get(&host, "trace"))
        .unwrap()
        .as_string()
        .unwrap();
    assert!(trace.contains(r#"["width",55296,true]"#), "{trace}");
    assert!(trace.contains(r#"["properties",66,2,true]"#), "{trace}");
    terminal.dispose().unwrap();
}

#[wasm_bindgen_test]
fn version_getter_errors_retain_identity_and_partial_registration() {
    let terminal = terminal();
    let view = terminal.unicode().unwrap();
    let api: JsValue = view.into();
    let result = register_throwing(&api, 0).unwrap();
    assert!(js_sys::Object::is(
        &get(&result, "failure"),
        &get(&result, "error")
    ));
    assert_eq!(get(&result, "reads"), JsValue::from(1));
    let value: JsValue = JSON::parse(r#"{"version":""}"#).unwrap();
    invoke(&api, "register", &value);
    Reflect::set(&api, &"activeVersion".into(), &"".into()).unwrap();
    let result = register_throwing(&api, 1).unwrap();
    assert!(js_sys::Object::is(
        &get(&result, "failure"),
        &get(&result, "error")
    ));
    assert_eq!(get(&result, "reads"), JsValue::from(2));
    let versions = js_sys::Array::from(&get(&api, "versions"));
    assert_eq!(
        versions.length(),
        3,
        "second getter failure retains the inserted provider"
    );
    assert_eq!(get(&api, "activeVersion"), JsValue::from(""));
    terminal.dispose().unwrap();
}

#[wasm_bindgen_test]
fn change_callbacks_observe_committed_utf16_state_and_can_reenter() {
    let terminal = terminal();
    let service = terminal.shared_native().unicode().unwrap();
    let api = terminal.unicode().unwrap();
    let observations = Rc::new(RefCell::new(Vec::new()));
    let view = Rc::new(api);
    let weak_view = Rc::downgrade(&view);
    let observed = Rc::clone(&observations);
    let subscription = service.on_change().subscribe(move |_| {
        let view = weak_view.upgrade().unwrap();
        let current = view.active_version();
        observed.borrow_mut().push(current.clone());
        if current == "11" {
            view.set_active_version("6".into()).unwrap();
        }
    });
    // Unicode11 registers without changing the selected provider.
    xterm_web::unicode_addons::Unicode11Addon::new()
        .activate(&terminal)
        .unwrap();
    view.set_active_version("11".into()).unwrap();
    view.set_active_version("6".into()).unwrap();
    assert_eq!(&*observations.borrow(), &["11", "6", "6"]);
    subscription.dispose();
    let failure = js_sys::Object::new();
    let thrown: JsValue = failure.clone().into();
    let subscription = service
        .on_change()
        .try_subscribe(move |_| Err(xterm_web::js_error::callback_error(thrown.clone())));
    let error = view.set_active_version("11".into()).unwrap_err();
    assert!(js_sys::Object::is(&error, &failure));
    assert_eq!(view.active_version(), js_sys::JsString::from("11"));
    subscription.dispose();
    terminal.dispose().unwrap();
}
