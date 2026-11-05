#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;

wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/font_metadata_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = metadataRejectionScenarios)]
    async fn metadata_rejections(create: &Function, bytes: &Uint8Array)
        -> Result<JsValue, JsValue>;
}

#[wasm_bindgen_test(async)]
async fn bad_metadata_rejects_loading_without_refresh_and_other_family_can_load() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let bytes = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    );
    let actual = metadata_rejections(create.as_ref().unchecked_ref(), &bytes)
        .await
        .unwrap();
    let fixture = js_sys::JSON::parse(include_str!("fixtures/font-metadata-addon.json")).unwrap();
    let expected = Reflect::get(&fixture, &"metadata".into()).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}
