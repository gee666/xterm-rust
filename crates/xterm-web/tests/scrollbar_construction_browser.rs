#![cfg(target_arch = "wasm32")]
//! Exact synchronous property reads on the real public Terminal.
use js_sys::{Array, Reflect, JSON};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/scrollbar_construction_cases.js")]
extern "C" {
    #[wasm_bindgen(js_name = scrollbarScenario)]
    fn scenario(index: u32) -> JsValue;
    #[wasm_bindgen(catch, js_name = scrollbarCapture)]
    fn capture(scenario: &JsValue, error: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = scrollbarExpected, thread_local_v2)]
    static EXPECTED: Array;
}
fn property(object: &JsValue, key: &str) -> JsValue {
    Reflect::get(object, &key.into()).unwrap()
}

#[wasm_bindgen_test]
fn full_terminal_scrollbar_getters_and_failure_identity_match_source() {
    EXPECTED.with(|expected| {
        assert_eq!(expected.length(), 8);
        for index in 0..expected.length() {
            let scenario = scenario(index);
            let terminal = Terminal::new(property(&scenario, "options")).unwrap();
            let parent: HtmlElement = property(&scenario, "parent").unchecked_into();
            let result = terminal.open(parent.clone().into()).and_then(|()| {
                if index == 5 {
                    property(&scenario, "reads")
                        .unchecked_into::<Array>()
                        .set_length(0);
                    Reflect::set(
                        &terminal.options(),
                        &"scrollbar".into(),
                        &property(&scenario, "update"),
                    )?;
                }
                Ok(())
            });
            let captured = capture(&scenario, &result.err().unwrap_or(JsValue::UNDEFINED));
            terminal.dispose().unwrap();
            parent.remove();
            assert_eq!(
                JSON::stringify(&captured.unwrap()).unwrap(),
                JSON::stringify(&expected.get(index)).unwrap(),
                "scrollbar scenario {index}"
            );
        }
    });
}
