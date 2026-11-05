#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/browser_mouse_builtin_cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = liveBuiltinMousePolicies)]
    fn policies(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = rawMouseFieldOrdering)]
    fn ordering(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = rawMouseFailures)]
    fn failures(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = mousePolicyReentryAndParsedModes)]
    async fn reentry(term: JsValue) -> Result<JsValue, JsValue>;
}

fn compare(name: &str, actual: JsValue) {
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&actual)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../results/mouse-builtin-methods/source-chromium.json"
    ))
    .unwrap();
    assert_eq!(actual, expected[name], "source case {name}");
}

fn terminal() -> JsValue {
    Terminal::new(JsValue::UNDEFINED).unwrap().into()
}

#[wasm_bindgen_test]
fn live_builtin_protocol_and_encoding_policies_match_source() {
    compare("liveBuiltinMousePolicies", policies(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn raw_property_reads_writes_and_coercion_order_match_source() {
    compare("rawMouseFieldOrdering", ordering(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn arbitrary_getter_setter_errors_keep_identity() {
    compare("rawMouseFailures", failures(terminal()).unwrap());
}

#[wasm_bindgen_test(async)]
async fn reentry_parser_modes_reset_and_disposal_use_live_native_state() {
    compare(
        "mousePolicyReentryAndParsedModes",
        reentry(terminal()).await.unwrap(),
    );
}
