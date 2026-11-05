#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_metadata_data.rs"]
mod corpus;
#[path = "../../xterm-addons/tests/support/font_kern_warning_assertions.rs"]
mod warnings;

#[wasm_bindgen_test]
fn source_kern_warning_corpus_matches_wasm() {
    warnings::assert_corpus();
    warnings::assert_reentry_and_earlier_errors();
    // Pin the shared JS font builder as a WASM snippet dependency and verify
    // that its appended descriptor produces the same constructor diagnostic.
    let original = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    );
    let mut payload = [0; 18];
    payload[1] = 1;
    payload[7] = 2;
    let bytes = with_kern(&original, &Uint8Array::from(payload.as_slice())).unwrap();
    let mut count = 0;
    xterm_addons::ligatures::font::Font::load_buffer_with_warnings(
        &bytes.to_vec(),
        Default::default(),
        |_| {
            count += 1;
            Ok::<(), JsValue>(())
        },
    )
    .unwrap();
    assert_eq!(count, 1);
}

#[wasm_bindgen(module = "/tests/support/font_metadata_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = withKern)]
    fn with_kern(bytes: &Uint8Array, payload: &Uint8Array) -> Result<Uint8Array, JsValue>;
}

#[wasm_bindgen(module = "/tests/support/font_kern_warning_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = kernWarningScenarios)]
    async fn kern_warnings(create: &Function, bytes: &Uint8Array) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen_test(async)]
async fn kern_warning_callbacks_preserve_receiver_errors_reentry_and_family_changes() {
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let bytes = Uint8Array::from(
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/UbuntuMono-Regular.ttf")
            .as_slice(),
    );
    let actual = kern_warnings(create.as_ref().unchecked_ref(), &bytes)
        .await
        .unwrap();
    let expected =
        js_sys::JSON::parse(include_str!("fixtures/font-kern-warning-addon.json")).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}

wasm_bindgen_test_configure!(run_in_browser);
