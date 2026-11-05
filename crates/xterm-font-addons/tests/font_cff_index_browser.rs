#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_cff_index_assertions.rs"]
mod assertions;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/font_cff_index_input.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = cffIndexInput)]
    fn input(base: &Uint8Array, case: &JsValue, encoded: &JsValue) -> Uint8Array;
}
#[wasm_bindgen(module = "/tests/support/font_cff_header_addon.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = cffHeaderAddonScenarios)]
    async fn addon_scenarios(create: &Function, inputs: &JsValue) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen_test]
fn source_index_corpus_and_independent_input_recipes_match_wasm() {
    assertions::assert_corpus();
    assertions::assert_header_regression();
    let fixture = assertions::fixture();
    let base = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf")
            .as_slice(),
    );
    for case in fixture["cases"].as_array().unwrap() {
        let mut encoded = fixture["woffTemplates"]["otf"].clone();
        if let Some(overrides) = case["encoded"].as_object() {
            encoded.as_object_mut().unwrap().extend(overrides.clone());
        }
        let encoded = js_sys::JSON::parse(&encoded.to_string()).unwrap();
        let raw = js_sys::JSON::parse(&case.to_string()).unwrap();
        assert_eq!(
            input(&base, &raw, &encoded).to_vec(),
            assertions::construct(case, &fixture),
            "input {}",
            case["name"]
        );
    }
}
#[wasm_bindgen_test(async)]
async fn actual_addon_index_rejection_caching_and_recovery() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let fixture = assertions::fixture();
    let cases = fixture["cases"].as_array().unwrap();
    let inputs = js_sys::Object::new();
    for (name, tag, woff, truncate) in [
        ("BadCff", "CFF ", false, Some(4)),
        ("BadCff2", "CFF2", true, Some(15353)),
        ("GoodCff", "CFF ", false, None),
        ("GoodCff2", "CFF2", true, None),
    ] {
        let case = cases
            .iter()
            .find(|c| {
                c["tag"] == tag
                    && c["woff"] == woff
                    && c["truncate"].as_u64() == truncate
                    && c["patches"].is_null()
                    && c["data"].is_null()
            })
            .unwrap();
        js_sys::Reflect::set(
            &inputs,
            &name.into(),
            &Uint8Array::from(assertions::construct(case, &fixture).as_slice()),
        )
        .unwrap();
    }
    let raw = addon_scenarios(create.as_ref().unchecked_ref(), inputs.as_ref())
        .await
        .unwrap();
    let serialized = js_sys::JSON::stringify(&raw).unwrap().as_string().unwrap();
    console_log!(
        "CFF INDEX actual addon raw errors/lifecycle: {}",
        serialized
    );
    let actual: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    let mut expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/font-cff-header-addon.json")).unwrap();
    // This is explicitly a remaining adapter gap, not a source-message equality
    // assertion. Source-native messages are retained for every engine separately.
    expected["debug"][1] =
        serde_json::json!({"name":"RangeError","message":"Font table offset is outside the input"});
    assert_eq!(actual, expected);
}
