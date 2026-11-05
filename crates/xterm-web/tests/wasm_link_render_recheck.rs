#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::terminal::Terminal;
wasm_bindgen_test_configure!(run_in_browser);
#[path = "linkifier/render_recheck.rs"]
mod render_recheck;

#[wasm_bindgen_test(async)]
async fn opened_terminal_rechecks_live_link_range_before_leaving_on_render() {
    let options = Object::new();
    Reflect::set(&options, &"cols".into(), &20.into()).unwrap();
    Reflect::set(&options, &"rows".into(), &4.into()).unwrap();
    let terminal: JsValue = Terminal::new(options.into()).unwrap().into();
    let program = Function::new_with_args(
        "terminal",
        include_str!("../../../scripts/link-render-recheck-trace.js"),
    );
    let pending: Promise = program
        .call1(&JsValue::UNDEFINED, &terminal)
        .unwrap()
        .dyn_into()
        .unwrap();
    let result = JsFuture::from(pending).await.unwrap();
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&result)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/link-render-recheck.json")).unwrap();
    assert_eq!(actual, expected);
}
