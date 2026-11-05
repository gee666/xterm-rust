#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn real_core_and_buffer_views_follow_parser_switch_resize_and_reset() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent: HtmlElement = document.create_element("div").unwrap().dyn_into().unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    let options = Object::new();
    Reflect::set(&options, &"cols".into(), &10.into()).unwrap();
    Reflect::set(&options, &"rows".into(), &4.into()).unwrap();
    let terminal = Terminal::new(options.into()).unwrap();
    terminal.open(parent.clone().into()).unwrap();
    let terminal: JsValue = terminal.into();
    let run = Function::new_with_args(
        "terminal",
        include_str!("../../../scripts/core-view-trace.js"),
    );
    let pending: Promise = run
        .call1(&JsValue::UNDEFINED, &terminal)
        .unwrap()
        .dyn_into()
        .unwrap();
    let result = JsFuture::from(pending).await;
    let dispose: Function = Reflect::get(&terminal, &"dispose".into())
        .unwrap()
        .dyn_into()
        .unwrap();
    dispose.call0(&terminal).unwrap();
    parent.remove();
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&result.unwrap())
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/core-views.json")).unwrap();
    assert_eq!(actual, expected);
}
