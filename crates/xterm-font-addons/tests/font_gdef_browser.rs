#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Reflect, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_metadata_data.rs"]
mod corpus;
#[path = "../../xterm-addons/tests/support/font_gdef_assertions.rs"]
mod gdef;

#[wasm_bindgen_test]
fn source_gdef_constructor_corpus_matches_wasm() {
    gdef::assert_corpus();
}

#[wasm_bindgen(module = "/tests/support/font_gdef_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = gdefLoadingScenario)]
    async fn loading(
        create: &Function,
        inputs: &JsValue,
        good: &Uint8Array,
    ) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen_test(async)]
async fn real_addon_preserves_gdef_warnings_alias_mutations_and_rejection_cache() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let fixture = gdef::fixture();
    let inputs = Object::new();
    for (mode, name) in [
        ("Warnings", "GDEF two class warnings SFNT"),
        ("Throw", "GDEF two class warnings SFNT"),
        ("Mutate", "GDEF class warning skips next pointer SFNT"),
        ("Reject", "GDEF invalid version SFNT"),
        (
            "Compressed",
            "GDEF large classes warning mutation compressed",
        ),
        ("LaterMeta", "GDEF warning breaks later META SFNT"),
    ] {
        let case = fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|c| c["name"] == name)
            .unwrap();
        let bytes = Uint8Array::from(gdef::input(&fixture, case).as_slice());
        Reflect::set(&inputs, &mode.into(), &bytes).unwrap();
    }
    let good = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    );
    let actual = loading(create.as_ref().unchecked_ref(), &inputs, &good)
        .await
        .unwrap();
    let expected = js_sys::JSON::parse(include_str!("fixtures/font-gdef-addon.json")).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}

wasm_bindgen_test_configure!(run_in_browser);
