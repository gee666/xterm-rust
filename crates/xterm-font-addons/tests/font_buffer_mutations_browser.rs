#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use wasm_bindgen_test::*;
use xterm_font_addons::LigaturesAddon;
#[path = "../../xterm-addons/tests/support/font_buffer_data.rs"]
mod corpus;

#[wasm_bindgen(module = "/tests/support/font_metadata_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = withKern)]
    fn with_kern(bytes: &Uint8Array, payload: &Uint8Array) -> Result<Uint8Array, JsValue>;
}
#[wasm_bindgen(module = "/tests/support/font_buffer_mutations.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = bufferMutationScenarios)]
    async fn mutations(create: &Function, inputs: &Array) -> Result<JsValue, JsValue>;
}

#[wasm_bindgen_test]
fn source_constructor_mutations_match_wasm() {
    corpus::assert_corpus();
    // Pin the nested shared builder import into the wasm-bindgen snippet set.
    assert_eq!(
        with_kern(
            &Uint8Array::from(&[0; 12][..]),
            &Uint8Array::new_with_length(0)
        )
        .unwrap()
        .length(),
        28
    );
}

#[wasm_bindgen_test(async)]
async fn live_local_font_buffer_mutations_match_source_addon() {
    let fixture = corpus::fixture();
    let inputs = Array::new();
    for item in fixture["constructors"].as_array().unwrap() {
        inputs.push(&Uint8Array::from(corpus::construct(item).as_slice()));
    }
    let create =
        Closure::wrap(
            Box::new(|options: JsValue| LigaturesAddon::new(options).map(JsValue::from))
                as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>,
        );
    let actual = mutations(create.as_ref().unchecked_ref(), &inputs)
        .await
        .unwrap();
    // Value serialization avoids property insertion-order differences from serde maps.
    let actual: serde_json::Value =
        serde_json::from_str(&String::from(js_sys::JSON::stringify(&actual).unwrap())).unwrap();
    assert_eq!(actual, fixture["addon"]);
}
wasm_bindgen_test_configure!(run_in_browser);
