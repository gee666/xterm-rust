#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_gsub_lookup_assertions.rs"]
mod assertions;
#[path = "../../xterm-addons/tests/support/font_metadata_data.rs"]
mod corpus;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/font_gsub_header_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = headerAddonCases)]
    async fn addon_cases(
        create: &Function,
        input: &Function,
        count: u32,
    ) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen_test]
fn source_gsub_lookup_corpus_matches_in_wasm() {
    assertions::assert_corpus();
}
#[wasm_bindgen_test(async)]
async fn actual_addon_loading_failure_and_cache() {
    let fixture = assertions::fixture();
    let count = u32::try_from(fixture["cases"].as_array().unwrap().len()).unwrap();
    let input = Closure::wrap(Box::new(move |index: u32| {
        Uint8Array::from(assertions::input(&fixture, &fixture["cases"][index as usize]).as_slice())
    }) as Box<dyn FnMut(u32) -> Uint8Array>);
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let actual = addon_cases(
        create.as_ref().unchecked_ref(),
        input.as_ref().unchecked_ref(),
        count,
    )
    .await
    .unwrap();
    let expected =
        js_sys::JSON::parse(include_str!("fixtures/font-gsub-lookup-addon.json")).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}
