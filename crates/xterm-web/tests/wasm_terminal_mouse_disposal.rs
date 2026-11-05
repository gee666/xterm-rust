#![cfg(target_arch = "wasm32")]

use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/browser_mouse_dispose_cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = directMouseDisposal)]
    fn direct(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = singleListenerDisposal)]
    fn single(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = multipleListenerDisposal)]
    fn multiple(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = removedListenerDisposal)]
    fn removed(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = addedListenerDisposal)]
    fn added(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = throwingListenerDisposal)]
    fn throwing(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = parsedModesAfterDisposal)]
    fn parsed(term: JsValue) -> Promise;
    #[wasm_bindgen(js_name = domRoutingAfterDisposal)]
    fn dom(term: JsValue) -> Promise;
}

fn terminal() -> JsValue {
    Terminal::new(JsValue::UNDEFINED).unwrap().into()
}

fn compare(name: &str, actual: JsValue) {
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../results/mouse-disposal/source-chromium.json"
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
fn direct_disposal_preserves_modes_methods_identity_and_retained_handles() {
    compare("directMouseDisposal", direct(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn single_listener_disposal_supports_reentrant_reset() {
    compare("singleListenerDisposal", single(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn cleared_listener_array_interrupts_reset_before_encoding_change() {
    compare("multipleListenerDisposal", multiple(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn removed_listener_snapshot_survives_disposal_of_current_array() {
    compare("removedListenerDisposal", removed(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn added_listener_snapshot_survives_disposal_of_current_array() {
    compare("addedListenerDisposal", added(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn listener_disposal_preserves_original_thrown_identity() {
    compare("throwingListenerDisposal", throwing(terminal()).unwrap());
}

#[wasm_bindgen_test(async)]
async fn parsed_modes_and_reset_remain_live_without_notifications() {
    compare(
        "parsedModesAfterDisposal",
        JsFuture::from(parsed(terminal())).await.unwrap(),
    );
}

#[wasm_bindgen_test(async)]
async fn direct_service_disposal_retains_real_dom_routing_and_wheel_hook() {
    compare(
        "domRoutingAfterDisposal",
        JsFuture::from(dom(terminal())).await.unwrap(),
    );
}
