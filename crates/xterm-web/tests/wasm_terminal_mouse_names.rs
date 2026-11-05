#![cfg(target_arch = "wasm32")]

use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/browser_mouse_name_cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = unknownMouseStringNames)]
    fn names(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = everyLoneMouseNameSurrogate)]
    fn surrogates(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = seededAndLongMouseNames)]
    fn generated(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = reentrantMouseNameFailure)]
    fn reentry(term: JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = parsedModesSurviveMouseNameErrors)]
    async fn parsed(term: JsValue) -> Result<JsValue, JsValue>;
}

fn compare(name: &str, actual: JsValue) {
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&actual)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value = serde_json::from_str(include_str!(
        "../../../results/mouse-name-errors/source-chromium.json"
    ))
    .unwrap();
    assert_eq!(actual, expected[name], "source case {name}");
}

fn terminal() -> JsValue {
    Terminal::new(JsValue::UNDEFINED).unwrap().into()
}

#[wasm_bindgen_test]
fn unknown_string_names_preserve_error_text_state_and_events() {
    compare("unknownMouseStringNames", names(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn all_2048_lone_surrogates_survive_both_mode_errors() {
    compare(
        "everyLoneMouseNameSurrogate",
        surrogates(terminal()).unwrap(),
    );
}

#[wasm_bindgen_test]
fn seeded_names_and_long_names_preserve_every_code_unit() {
    compare("seededAndLongMouseNames", generated(terminal()).unwrap());
}

#[wasm_bindgen_test]
fn listener_reentry_keeps_error_identity_and_committed_modes() {
    compare("reentrantMouseNameFailure", reentry(terminal()).unwrap());
}

#[wasm_bindgen_test(async)]
async fn parsed_modes_and_reset_survive_rejected_names() {
    compare(
        "parsedModesSurviveMouseNameErrors",
        parsed(terminal()).await.unwrap(),
    );
}
