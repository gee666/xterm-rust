#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_hint_tables.rs"]
mod corpus;

#[wasm_bindgen(module = "/tests/support/font_hint_tables.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = buildHintFont)]
    fn build(base: &Uint8Array, change: &JsValue) -> Result<Uint8Array, JsValue>;
    #[wasm_bindgen(catch, js_name = hintAddonScenarios)]
    async fn addon_scenarios(
        create: &Function,
        base: &Uint8Array,
        cases: &JsValue,
    ) -> Result<JsValue, JsValue>;
}
fn base() -> Uint8Array {
    Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    )
}
fn fixture() -> JsValue {
    js_sys::JSON::parse(include_str!(
        "../../xterm-addons/tests/fixtures/font-hint-tables.json"
    ))
    .unwrap()
}
#[wasm_bindgen_test]
fn hint_constructor_outcomes_and_independent_input_builders_match_source() {
    corpus::assert_corpus();
    let fixture = corpus::fixture();
    for case in fixture["cases"].as_array().unwrap() {
        let value = js_sys::JSON::parse(&case.to_string()).unwrap();
        assert_eq!(
            corpus::construct(case),
            build(&base(), &value).unwrap().to_vec(),
            "{}",
            case["name"]
        );
    }
}
#[wasm_bindgen_test(async)]
async fn hint_constructor_rejections_and_successes_drive_real_addon_loading() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let fixture = fixture();
    let cases = js_sys::Reflect::get(&fixture, &"cases".into()).unwrap();
    let actual = addon_scenarios(create.as_ref().unchecked_ref(), &base(), &cases)
        .await
        .unwrap();
    let expected = js_sys::Reflect::get(&fixture, &"addon".into()).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}
wasm_bindgen_test_configure!(run_in_browser);
