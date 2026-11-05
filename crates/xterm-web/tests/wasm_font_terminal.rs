#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Promise, Reflect};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{LigaturesAddon, Terminal};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/support/font_terminal.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = terminalJoinerTrace)]
    fn trace(terminal: JsValue, addon: JsValue) -> Promise;
    #[wasm_bindgen(js_name = terminalJoinerMutation)]
    fn mutate(terminal: JsValue) -> Promise;
}
fn terminal() -> Terminal {
    let options = Object::new();
    for (key, value) in [("cols", 10.0), ("rows", 2.0)] {
        Reflect::set(&options, &key.into(), &value.into()).unwrap();
    }
    Terminal::new(options.into()).unwrap()
}
async fn result(promise: Promise) -> Value {
    let value = JsFuture::from(promise).await.unwrap();
    serde_json::from_str(&value.as_string().unwrap()).unwrap()
}
#[wasm_bindgen_test]
fn joiner_registration_and_deregistration_require_open() {
    let terminal = terminal();
    let handler = Function::new_no_args("return []");
    for error in [
        terminal.register_character_joiner(handler).unwrap_err(),
        terminal.deregister_character_joiner(0.0).unwrap_err(),
    ] {
        assert_eq!(
            js_sys::Error::from(error).message().as_string().unwrap(),
            "Terminal must be opened first"
        );
    }
    terminal.dispose().unwrap();
}
#[wasm_bindgen_test(async)]
async fn public_joiners_and_ligatures_addon_change_actual_dom_spans() {
    let addon = LigaturesAddon::new(JsValue::UNDEFINED).unwrap();
    let trace = result(trace(terminal().into(), addon.into())).await;
    // The DOM renderer groups adjacent unjoined cells with equal attributes.
    assert_eq!(trace["before"], json!(["a===b"]));
    assert_eq!(trace["joined"], json!(["a", "===", "b"]));
    assert_eq!(trace["after"], trace["before"]);
    assert_eq!(trace["fallback"], trace["joined"]);
    assert_eq!(trace["disposed"], trace["before"]);
    assert_eq!(trace["style"], trace["expectedStyle"]);
    assert_eq!(trace["styleAfter"], "");
    assert_eq!(trace["removedIsUndefined"], true);
    let calls = trace["calls"].as_array().unwrap();
    assert!(!calls.is_empty());
    assert!(calls.iter().all(|text| text == "a===b"));
}
#[wasm_bindgen_test(async)]
async fn joiner_can_remove_and_add_handlers_during_real_rendering() {
    let trace = result(mutate(terminal().into())).await;
    assert_eq!(trace["spans"], json!(["a", "===", "b"]));
    let calls = trace["calls"].as_array().unwrap();
    assert_eq!(calls[0], json!(["first", "a===b"]));
    assert_eq!(calls[1], json!(["third", "a===b"]));
    assert!(calls.iter().all(|call| call[0] != "second"));
}
