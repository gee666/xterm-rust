#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/browser_mouse_wheel_method_cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = directWheelValues)]
    fn values(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = borrowedWheelMethods)]
    fn borrowed(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = wheelAccessorOrder)]
    fn accessors(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = sharedWheelRouting)]
    fn routing(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = wheelMethodReentry)]
    fn reentry(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = wheelMethodOverrides)]
    fn overrides(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = wheelMethodPrototype)]
    fn prototype(term: JsValue) -> Result<JsValue, JsValue>;
}

fn compare(name: &str, run: impl FnOnce(JsValue) -> Result<JsValue, JsValue>) {
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let actual = run(terminal.into()).unwrap();
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&actual)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../results/mouse-wheel-methods/source-chromium.json"
    ))
    .unwrap();
    assert_eq!(actual, expected[name], "source case {name}");
}

#[wasm_bindgen_test]
fn raw_handlers_results_and_errors() {
    compare("directWheelValues", values);
}
#[wasm_bindgen_test]
fn borrowed_receivers_and_inherited_properties() {
    compare("borrowedWheelMethods", borrowed);
}
#[wasm_bindgen_test]
fn accessor_order_and_errors() {
    compare("wheelAccessorOrder", accessors);
}
#[wasm_bindgen_test]
fn public_internal_and_dom_share_handler_across_lifecycle() {
    compare("sharedWheelRouting", routing);
}
#[wasm_bindgen_test]
fn recursive_replacement_and_reset() {
    compare("wheelMethodReentry", reentry);
}
#[wasm_bindgen_test]
fn live_method_overrides_and_exceptions() {
    compare("wheelMethodOverrides", overrides);
}
#[wasm_bindgen_test]
fn prototype_identity_descriptors_and_overrides() {
    compare("wheelMethodPrototype", prototype);
}
