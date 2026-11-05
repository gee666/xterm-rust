// Copyright (c) 2022-2026 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Object, Promise, Reflect};
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::webgl::device_pixel_observer::observe_device_pixel_dimensions;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/support/device_pixel_observer.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = observerTraces)]
    fn traces(factory: &Function) -> Result<String, JsValue>;
    #[wasm_bindgen(catch, js_name = oracleTraces)]
    fn oracle_traces() -> Result<String, JsValue>;
    #[wasm_bindgen(js_name = nativeObserver)]
    fn native_observer(factory: &Function) -> Promise;
    #[wasm_bindgen(js_name = oracleNative)]
    fn oracle_native() -> Promise;
    #[wasm_bindgen(js_name = dropProbe)]
    fn drop_probe() -> JsValue;
}
fn factory() -> Function {
    Closure::wrap(Box::new(
        move |element: JsValue, parent: JsValue, callback: Function| {
            let observer = Rc::new(observe_device_pixel_dimensions(
                element.unchecked_ref(),
                parent.unchecked_ref(),
                &callback,
            )?);
            let dispose = Closure::wrap(
                Box::new(move || observer.dispose()) as Box<dyn Fn() -> Result<(), JsValue>>
            );
            let object = Object::new();
            Reflect::set(&object, &"dispose".into(), &dispose.into_js_value())?;
            Ok(JsValue::from(object))
        },
    )
        as Box<dyn Fn(JsValue, JsValue, Function) -> Result<JsValue, JsValue>>)
    .into_js_value()
    .unchecked_into()
}

#[wasm_bindgen_test]
fn observer_lifecycle_and_live_getters_match_unchanged_source() {
    let expected = oracle_traces().unwrap();
    let actual = traces(&factory()).unwrap();
    let cases: serde_json::Value = serde_json::from_str(&expected).unwrap();
    assert_eq!(cases.as_array().unwrap().len(), 25);
    assert_eq!(actual, expected);
}

#[wasm_bindgen_test]
fn rust_drop_disconnects_once_and_releases_callback_state() {
    let probe = drop_probe();
    let property = |name: &str| Reflect::get(&probe, &name.into()).unwrap();
    let element = property("element");
    let parent = property("parent");
    let callback = property("callback").unchecked_into();
    let observer =
        observe_device_pixel_dimensions(element.unchecked_ref(), parent.unchecked_ref(), &callback)
            .unwrap();
    let deliver: Function = property("deliver").unchecked_into();
    let snapshot: Function = property("snapshot").unchecked_into();
    deliver.call0(&probe).unwrap();
    drop(observer);
    deliver.call0(&probe).unwrap();
    assert_eq!(
        snapshot.call0(&probe).unwrap().as_string().unwrap(),
        r#"{"disconnects":1,"calls":1}"#
    );
}

#[wasm_bindgen_test]
async fn actual_parent_window_resize_delivery_matches_unchanged_source() {
    let expected = JsFuture::from(oracle_native()).await.unwrap();
    let actual = JsFuture::from(native_observer(&factory())).await.unwrap();
    assert_eq!(actual, expected);
    let result: serde_json::Value = serde_json::from_str(&actual.as_string().unwrap()).unwrap();
    assert_eq!(result["constructors"], 1);
    assert_eq!(
        result["disconnects"],
        if result["sizes"][0].is_null() { 1 } else { 2 }
    );
    assert!(!result["sizes"].as_array().unwrap().is_empty());
    if result["sizes"][0].is_null() {
        assert_eq!(result["callbacks"], serde_json::json!([]));
    } else {
        assert_eq!(result["sizes"], result["callbacks"]);
        assert_eq!(result["callbacks"].as_array().unwrap().len(), 2);
        assert_ne!(result["callbacks"][0][0], result["callbacks"][1][0]);
    }
}
