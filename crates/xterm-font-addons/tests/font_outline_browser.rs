#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Reflect, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_metadata_data.rs"]
mod corpus;
#[path = "../../xterm-addons/tests/support/font_outline_assertions.rs"]
mod outline;

#[wasm_bindgen_test]
fn source_truetype_constructor_boundaries_match_wasm() {
    outline::assert_corpus();
}

#[wasm_bindgen(module = "/tests/support/font_outline_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = outlineLoadingScenario)]
    async fn loading(
        create: &Function,
        inputs: &JsValue,
        good: &Uint8Array,
    ) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen_test(async)]
async fn real_addon_preserves_outline_rejection_cache_and_recovery() {
    let create = Closure::wrap(
        Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
            as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
    );
    let fixture = outline::fixture();
    let inputs = Object::new();
    for (mode, name) in [
        ("Reject", "loca EOF undefined/0/0 SFNT"),
        ("CompressedReject", "loca EOF undefined/0/2 WOFF"),
        ("LazyOffsets", "loca arbitrary offsets remain lazy SFNT"),
        ("EmptyGlyf", "glyf empty lazy body WOFF"),
    ] {
        let case = fixture["cases"]
            .as_array().unwrap().iter()
            .find(|c| c["name"] == name).unwrap();
        let bytes = Uint8Array::from(outline::input(&fixture, case).as_slice());
        Reflect::set(&inputs, &mode.into(), &bytes).unwrap();
    }
    let good = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    );
    let actual = loading(create.as_ref().unchecked_ref(), &inputs, &good).await.unwrap();
    let expected = js_sys::JSON::parse(include_str!("fixtures/font-outline-addon.json")).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}

wasm_bindgen_test_configure!(run_in_browser);
