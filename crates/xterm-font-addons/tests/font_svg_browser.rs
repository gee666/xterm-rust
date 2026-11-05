#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_metadata_data.rs"]
mod corpus;
#[path = "../../xterm-addons/tests/support/font_svg_assertions.rs"]
mod svg;

#[wasm_bindgen_test]
fn source_svg_constructor_corpus_matches_wasm() {
    svg::assert_corpus();
}

#[wasm_bindgen(module = "/tests/support/font_svg_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = svgLoadingScenario)]
    async fn svg_loading(create: &Function, bytes: &Uint8Array) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen_test(async)]
async fn malformed_svg_retains_fallback_without_refresh_and_caches_rejection() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let bytes = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    );
    let actual = svg_loading(create.as_ref().unchecked_ref(), &bytes)
        .await
        .unwrap();
    let expected = js_sys::JSON::parse(include_str!("fixtures/font-svg-addon.json")).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}

wasm_bindgen_test_configure!(run_in_browser);
