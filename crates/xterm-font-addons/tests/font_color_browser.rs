#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_color_assertions.rs"]
mod assertions;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/support/font_color_addon.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = colorAddonScenarios)]
    async fn addon_scenarios(
        create: &Function,
        bytes: &Uint8Array,
        insert: &Function,
        payload: &Function,
    ) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen(module = "/tests/support/font_color_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = colorPayload)]
    fn color_payload(tag: &str, options: JsValue) -> Uint8Array;
}
#[wasm_bindgen(module = "/tests/support/font_metadata_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = withKern)]
    fn insert_table(bytes: &Uint8Array, payload: &Uint8Array) -> Uint8Array;
}
#[wasm_bindgen_test]
fn source_color_corpus_matches_wasm() {
    assertions::assert_corpus();
}
#[wasm_bindgen_test(async)]
async fn color_load_rejections_cache_diagnostics_and_allow_recovery() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let bytes = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    );
    let insert = Closure::wrap(Box::new(|bytes: Uint8Array, payload: Uint8Array| {
        insert_table(&bytes, &payload)
    })
        as Box<dyn FnMut(Uint8Array, Uint8Array) -> Uint8Array>);
    let payload =
        Closure::wrap(
            Box::new(|tag: String, options: JsValue| color_payload(&tag, options))
                as Box<dyn FnMut(String, JsValue) -> Uint8Array>,
        );
    let actual = addon_scenarios(
        create.as_ref().unchecked_ref(),
        &bytes,
        insert.as_ref().unchecked_ref(),
        payload.as_ref().unchecked_ref(),
    )
    .await
    .unwrap();
    let expected = js_sys::JSON::parse(include_str!("fixtures/font-color-addon.json")).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}
