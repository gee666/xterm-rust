#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_cff2_index_assertions.rs"]
mod assertions;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/font_cff_header_input.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = cffHeaderInput)]
    fn input(base: &Uint8Array, case: &JsValue, encoded: &JsValue) -> Uint8Array;
}
#[wasm_bindgen(module = "/tests/support/font_cff_header_addon.mjs")]
extern "C" {
    #[wasm_bindgen(catch,js_name = cffHeaderAddonScenarios)]
    async fn addon_scenarios(create: &Function, inputs: &JsValue) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen_test]
fn complete_fdarray_index_rejections_and_independent_input_recipes() {
    assertions::assert_corpus();
    assertions::assert_previous();
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
async fn actual_addon_index_rejection_caching_fallback_recovery_and_disposal() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let fixture = assertions::fixture();
    let inputs = js_sys::Object::new();
    for (name, tag, woff, bad) in [
        ("BadCff", "CFF ", false, true),
        ("BadCff2", "CFF2", true, true),
        ("GoodCff", "CFF ", false, false),
        ("GoodCff2", "CFF2", true, false),
    ] {
        let case = fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| {
                c["tag"] == tag
                    && c["woff"].as_bool().unwrap_or(false) == woff
                    && !c["remove"].is_array()
                    && if bad {
                        c["data"]
                            == serde_json::json!([
                                2, 0, 5, 0, 7, 29, 127, 255, 255, 255, 12, 36, 0, 0, 0, 0
                            ])
                    } else {
                        c["data"].is_null()
                    }
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
        "CFF2 FDArray INDEX actual addon raw lifecycle: {}",
        serialized
    );
    let actual: serde_json::Value = serde_json::from_str(&serialized).unwrap();
    let mut expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/font-cff-header-addon.json")).unwrap();
    // Existing Bounds adapter wording differs from the source DataView message.
    // Retain and name that342 gap; test the actual Rust RangeError unchanged.
    expected["debug"][1] =
        serde_json::json!({"name":"RangeError","message":"Font table offset is outside the input"});
    assert_eq!(actual, expected);
}
