#![cfg(target_arch = "wasm32")]
use js_sys::{Object, Promise, Reflect, Uint8Array};
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::{LigaturesAddon, Terminal};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/support/font_shaping.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = terminalShapedFont)]
    fn trace(terminal: JsValue, addon: JsValue, bytes: Uint8Array) -> Promise;
}

#[wasm_bindgen_test(async)]
async fn loaded_font_shapes_actual_terminal_with_no_fallback_patterns() {
    let options = Object::new();
    for (key, value) in [("cols", 12.0), ("rows", 2.0)] {
        Reflect::set(&options, &key.into(), &value.into()).unwrap();
    }
    Reflect::set(&options, &"fontFamily".into(), &"Fira Code".into()).unwrap();
    let terminal = Terminal::new(options.into()).unwrap();
    let addon_options = Object::new();
    Reflect::set(
        &addon_options,
        &"fallbackLigatures".into(),
        &js_sys::Array::new(),
    )
    .unwrap();
    let addon = LigaturesAddon::new(addon_options.into()).unwrap();
    let font =
        include_bytes!("../../xterm-addons/tests/fixtures/ligature-fonts/FiraCode-Regular.otf");
    let value = JsFuture::from(trace(
        terminal.into(),
        addon.into(),
        Uint8Array::from(&font[..]),
    ))
    .await
    .unwrap();
    let result: Value = serde_json::from_str(&value.as_string().unwrap()).unwrap();
    assert_eq!(result["fontStatus"], "loaded");
    assert_eq!(result["fontReady"], true);
    assert_eq!(result["blobReads"], 1);
    assert_eq!(result["before"], json!(["a===b"]));
    assert_eq!(result["loaded"], json!(["a", "===", "b"]));
    assert_eq!(result["disposed"], result["before"]);
    assert!(result["computedFamily"]
        .as_str()
        .unwrap()
        .contains("Fira Code"));
    assert_eq!(result["computedFeatures"], result["expectedFeatures"]);
}
