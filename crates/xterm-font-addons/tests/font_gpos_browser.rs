#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_metadata_data.rs"]
mod corpus;
#[path = "../../xterm-addons/tests/support/font_gpos_assertions.rs"]
mod gpos;

#[wasm_bindgen(module = "/tests/support/font_gpos_input.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = buildGposFont)]
    fn build(base: &Uint8Array, fixture: &JsValue, item: &JsValue) -> Result<Uint8Array, JsValue>;
}
#[wasm_bindgen(module = "/tests/support/font_gpos_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = gposAddonCases)]
    async fn addon_cases(
        create: &Function,
        input: &Function,
        cases: &JsValue,
    ) -> Result<JsValue, JsValue>;
}
fn fixture() -> JsValue {
    js_sys::JSON::parse(include_str!(
        "../../xterm-addons/tests/fixtures/font-gpos.json"
    ))
    .unwrap()
}
fn base() -> Uint8Array {
    Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    )
}
#[wasm_bindgen_test]
fn gpos_constructor_outcomes_and_independent_inputs_match_source() {
    gpos::assert_corpus();
    let data = gpos::fixture();
    let fixture = fixture();
    let base = base();
    for case in data["cases"].as_array().unwrap() {
        let item = js_sys::JSON::parse(&case.to_string()).unwrap();
        assert_eq!(
            gpos::input(&data, case),
            build(&base, &fixture, &item).unwrap().to_vec(),
            "{}",
            case["name"]
        );
    }
}
#[wasm_bindgen_test(async)]
async fn gpos_constructor_drives_real_addon_loading_warnings_and_cache() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let fixture = fixture();
    let cases = js_sys::Reflect::get(&fixture, &"cases".into()).unwrap();
    let data = gpos::fixture();
    let input = Closure::wrap(Box::new(move |index: u32| {
        Uint8Array::from(
            gpos::input(&data, &data["cases"][usize::try_from(index).unwrap()]).as_slice(),
        )
    }) as Box<dyn FnMut(u32) -> Uint8Array>);
    let actual = addon_cases(
        create.as_ref().unchecked_ref(),
        input.as_ref().unchecked_ref(),
        &cases,
    )
    .await
    .unwrap();
    let expected = js_sys::JSON::parse(include_str!("fixtures/font-gpos-addon.json")).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}
wasm_bindgen_test_configure!(run_in_browser);
