#![cfg(target_arch = "wasm32")]

use js_sys::{Object, Promise, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/browser_marker_cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = markerValidationCase)]
    fn validation_case(terminal: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = markerLifecycleCase)]
    fn lifecycle_case(terminal: JsValue) -> Promise;
    #[wasm_bindgen(catch, js_name = markerResourcesCase)]
    fn resources_case(terminal: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = markerBufferCase)]
    fn buffer_case(terminal: JsValue) -> Promise;
    #[wasm_bindgen(js_name = markerEventPropertyCase)]
    fn event_property_case(terminal: JsValue) -> Promise;
}

fn terminal() -> JsValue {
    let options = Object::new();
    Reflect::set(&options, &"rows".into(), &3.into()).unwrap();
    Reflect::set(&options, &"scrollback".into(), &2.into()).unwrap();
    // No proposed-API opt in: source marker APIs do not require it.
    Terminal::new(options.into()).unwrap().into()
}

fn assert_capture(actual: JsValue, expected: serde_json::Value) {
    let json = js_sys::JSON::stringify(&actual)
        .unwrap()
        .as_string()
        .unwrap();
    let actual: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(actual, expected);
}

#[wasm_bindgen_test]
fn marker_offset_validation_preserves_original_values_hints_and_errors() {
    assert_capture(
        validation_case(terminal()).unwrap(),
        serde_json::json!({
            "lines": [0, 0, 0, 1, 2, -2, "02", "0", "0 2 ", 9_007_199_254_740_992_u64],
            "errors": vec!["This API only accepts integers"; 6],
            "hints": ["number", "number", "default"], "objectLine": "07"
        }),
    );
}

#[wasm_bindgen_test(async)]
async fn marker_handles_follow_native_trim_and_source_disposal_dispatch() {
    assert_capture(
        JsFuture::from(lifecycle_case(terminal())).await.unwrap(),
        serde_json::json!({
            "seen": [[true, true, true, -1]], "disposed": [true, true, true],
            "lines": [-1, -1, -1], "remaining": 0
        }),
    );
}

#[wasm_bindgen_test]
fn marker_resources_preserve_live_methods_receiver_errors_and_traversal() {
    assert_capture(
        resources_case(terminal()).unwrap(),
        serde_json::json!({
            "log": ["event", "resource", "resource", "event-added", "appended", "appended", "changed"],
            "gets": 2, "errors": [true, true, true, true]
        }),
    );
}

#[wasm_bindgen_test(async)]
async fn marker_event_is_a_writable_own_property_independent_of_native_disposal() {
    assert_capture(
        JsFuture::from(event_property_case(terminal()))
            .await
            .unwrap(),
        serde_json::json!({
            "descriptor": [true, true, true], "gets": 1,
            "events": ["replacement", "first", "resource", "trim", "saved"], "remaining": 0
        }),
    );
}

#[wasm_bindgen_test(async)]
async fn marker_registration_captures_buffer_before_default_hint_coercion() {
    assert_capture(
        JsFuture::from(buffer_case(terminal())).await.unwrap(),
        serde_json::json!({
            "before": 1, "alternateDisposed": true, "capturedLine": 3,
            "capturedInCurrentBuffer": false, "hints": ["number", "number", "default"]
        }),
    );
}
