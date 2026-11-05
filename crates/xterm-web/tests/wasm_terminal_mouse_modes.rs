#![cfg(target_arch = "wasm32")]

use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/browser_mouse_mode_cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = initialMouseModes)]
    fn initial(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = mouseModeSetters)]
    fn setters(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = mouseModeReentry)]
    fn reentry(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = parsedMouseModes)]
    fn parsed(term: JsValue) -> Promise;
    #[wasm_bindgen(js_name = mouseDomRouting)]
    fn dom_routing(term: JsValue) -> Promise;
}

fn terminal() -> JsValue {
    // Internal service access, like the source, needs no proposed API opt in.
    Terminal::new(JsValue::UNDEFINED).unwrap().into()
}

fn compare(name: &str, actual: JsValue) {
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../results/mouse-modes/source-chromium.json"
    ))
    .unwrap();
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&actual)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    assert_eq!(actual, expected[name], "source case {name}");
}

#[wasm_bindgen_test]
fn stable_service_identity_and_initial_modes_before_open() {
    compare("initialMouseModes", initial(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn builtin_string_setters_errors_events_and_subscription_disposal() {
    compare("mouseModeSetters", setters(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn protocol_events_support_nested_mode_changes_and_original_thrown_identity() {
    compare("mouseModeReentry", reentry(terminal()).unwrap());
}

#[wasm_bindgen_test(async)]
async fn mode_setters_control_real_dom_tracking_and_native_output() {
    compare(
        "mouseDomRouting",
        JsFuture::from(dom_routing(terminal())).await.unwrap(),
    );
}

#[wasm_bindgen_test(async)]
async fn parser_modes_and_terminal_reset_share_the_retained_service() {
    compare(
        "parsedMouseModes",
        JsFuture::from(parsed(terminal())).await.unwrap(),
    );
}
