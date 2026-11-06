#![cfg(target_arch = "wasm32")]
//! Shared public trace on actual exported Rust Terminal instances, not a facade.
use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

fn terminal() -> JsValue {
    let options = Object::new();
    for (key, value) in [("cols", 20), ("rows", 4)] {
        Reflect::set(&options, &key.into(), &value.into()).unwrap();
    }
    Terminal::new(options.into()).unwrap().into()
}
#[wasm_bindgen_test(async)]
async fn exported_decoration_registration_shares_native_geometry_events_and_lifecycle() {
    let program = include_str!("../../../scripts/public-decoration-trace.js");
    let function = Function::new_with_args("terminal,other", program);
    let promise = function
        .call2(&JsValue::UNDEFINED, &terminal(), &terminal())
        .unwrap();
    let trace = JsFuture::from(promise.unchecked_into::<js_sys::Promise>())
        .await
        .unwrap();
    assert_eq!(Array::from(&trace).length(), 14);
    let actual: serde_json::Value = serde_json::from_str(
        &js_sys::JSON::stringify(&trace)
            .unwrap()
            .as_string()
            .unwrap(),
    )
    .unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/public-decoration.json")).unwrap();
    assert_eq!(actual, expected);
}
