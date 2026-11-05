#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_cff_header_assertions.rs"]
mod assertions;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/font_cff_header_input.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = cffHeaderInput)]
    fn input(base: &Uint8Array, case: &JsValue, encoded: &JsValue) -> Uint8Array;
}
#[wasm_bindgen(module = "/tests/support/font_cff_header_addon.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = cffHeaderAddonScenarios)]
    async fn addon_scenarios(create: &Function, inputs: &JsValue) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen_test]
fn source_header_corpus_and_independent_inputs_match_wasm() {
    assertions::assert_corpus();
    let fixture = assertions::fixture();
    let otf = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf")
            .as_slice(),
    );
    let ttf = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    );
    for case in fixture["cases"].as_array().unwrap() {
        let key = if case["base"] == "ttf" { "ttf" } else { "otf" };
        let mut encoded = fixture["woffTemplates"][key].clone();
        if let Some(overrides) = case["encoded"].as_object() {
            encoded.as_object_mut().unwrap().extend(overrides.clone());
        }
        let encoded = js_sys::JSON::parse(&encoded.to_string()).unwrap();
        let raw = js_sys::JSON::parse(&case.to_string()).unwrap();
        let actual = input(if key == "ttf" { &ttf } else { &otf }, &raw, &encoded);
        assert_eq!(
            actual.to_vec(),
            assertions::construct(case, &fixture),
            "input {}",
            case["name"]
        );
    }
}
#[wasm_bindgen_test(async)]
async fn actual_addon_rejects_headers_caches_failures_and_recovers() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let inputs = js_sys::Object::new();
    let fixture = assertions::fixture();
    let bad = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["tag"] == "CFF "
                && c["data"] == serde_json::json!([3, 7])
                && c["remove"] == serde_json::json!(["hmtx"])
                && c["woff"] == false
        })
        .unwrap();
    let bad2 = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["tag"] == "CFF2"
                && c["data"] == serde_json::json!([3, 7])
                && c["remove"] == serde_json::json!(["hmtx"])
                && c["woff"] == false
        })
        .unwrap();
    let good = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["tag"] == "CFF " && c["header"] == serde_json::json!([1, 0, 4]) && c["woff"] == false
        })
        .unwrap();
    let good2 = fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| {
            c["tag"] == "CFF2" && c["header"] == serde_json::json!([1, 0, 4]) && c["woff"] == true
        })
        .unwrap();
    for (name, case) in [
        ("BadCff", bad),
        ("BadCff2", bad2),
        ("GoodCff", good),
        ("GoodCff2", good2),
    ] {
        js_sys::Reflect::set(
            &inputs,
            &name.into(),
            &Uint8Array::from(assertions::construct(case, &fixture).as_slice()),
        )
        .unwrap();
    }
    let actual = addon_scenarios(create.as_ref().unchecked_ref(), inputs.as_ref())
        .await
        .unwrap();
    let expected =
        js_sys::JSON::parse(include_str!("fixtures/font-cff-header-addon.json")).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}
