// Lookup and type9 suites use the same real addon lifecycle and byte builder.
use js_sys::{Function, Uint8Array};
use wasm_bindgen::{closure::Closure, prelude::*};
use xterm_font_addons::LigaturesAddon;
#[wasm_bindgen(module = "/tests/support/font_gsub_header_scenarios.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = headerAddonCases)]
    async fn addon_cases(
        create: &Function,
        input: &Function,
        count: u32,
    ) -> Result<JsValue, JsValue>;
}

pub async fn assert_loading(fixture: serde_json::Value, expected: &str) {
    let count = u32::try_from(fixture["cases"].as_array().unwrap().len()).unwrap();
    let input = Closure::wrap(Box::new(move |index: u32| {
        Uint8Array::from(
            super::assertions::input(&fixture, &fixture["cases"][index as usize]).as_slice(),
        )
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
    let expected = js_sys::JSON::parse(expected).unwrap();
    assert_eq!(
        js_sys::JSON::stringify(&actual).unwrap(),
        js_sys::JSON::stringify(&expected).unwrap()
    );
}
