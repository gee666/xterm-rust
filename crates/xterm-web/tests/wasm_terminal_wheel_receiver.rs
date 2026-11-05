#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/browser_wheel_receiver_cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = liveWheelReceiver)]
    fn live(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = wheelReentryAndReplacement)]
    fn reentry(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = wheelResultsAndThrows)]
    fn results(term: JsValue) -> Result<JsValue, JsValue>;
}

fn compare(name: &str, case: fn(JsValue) -> Result<JsValue, JsValue>) {
    let term = Terminal::new(JsValue::UNDEFINED).unwrap();
    let actual = case(term.into()).unwrap();
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&actual)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../results/wheel-receiver/source-chromium.json"
    ))
    .unwrap();
    assert_eq!(actual, expected[name], "source case {name}");
}

#[wasm_bindgen_test]
fn wheel_callbacks_observe_stable_live_service_and_original_event() {
    compare("liveWheelReceiver", live);
}

#[wasm_bindgen_test]
fn wheel_callback_reentry_replaces_hook_without_borrow_or_receiver_change() {
    compare("wheelReentryAndReplacement", reentry);
}

#[wasm_bindgen_test]
fn strict_false_veto_and_arbitrary_thrown_identities_survive_dom_dispatch() {
    compare("wheelResultsAndThrows", results);
}
