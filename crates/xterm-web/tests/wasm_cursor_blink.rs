// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
#![cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::webgl::cursor_blink::CursorBlinkStateManager;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/cursor_blink_cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runCases)]
    fn run_cases(factory: &js_sys::Function) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = sourceFactory)]
    fn source_factory(service: JsValue, render: js_sys::Function) -> JsValue;
    #[wasm_bindgen(catch, js_name = nativeDriver)]
    async fn native_driver(factory: &js_sys::Function) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = dropDriver)]
    fn drop_driver(factory: &js_sys::Function) -> Result<JsValue, JsValue>;
}

type Factory = Closure<dyn Fn(JsValue, js_sys::Function) -> Result<JsValue, JsValue>>;
fn factory() -> Factory {
    Closure::new(move |service, render: js_sys::Function| {
        let callback = Rc::new(move || render.call0(&JsValue::UNDEFINED).map(|_| ()));
        let manager = Rc::new(RefCell::new(Some(CursorBlinkStateManager::new(
            callback, service,
        )?)));
        let object = js_sys::Object::new();
        for name in [
            "visible", "paused", "restart", "pause", "resume", "dispose", "drop",
        ] {
            let manager = Rc::clone(&manager);
            let callback = Closure::<dyn Fn() -> Result<JsValue, JsValue>>::new(move || {
                if name == "drop" {
                    let old = manager.borrow_mut().take();
                    drop(old);
                    return Ok(JsValue::UNDEFINED);
                }
                // Release the facade borrow before callbacks reenter it.
                let live = manager
                    .borrow()
                    .as_ref()
                    .cloned()
                    .ok_or_else(|| JsValue::from_str("dropped manager"))?;
                match name {
                    "visible" => return Ok(live.is_cursor_visible().into()),
                    "paused" => return Ok(live.is_paused().into()),
                    "restart" => live.restart_blink_animation()?,
                    "pause" => live.pause()?,
                    "resume" => live.resume()?,
                    "dispose" => live.dispose()?,
                    _ => unreachable!("method names are the fixed list above"),
                }
                Ok(JsValue::UNDEFINED)
            });
            js_sys::Reflect::set(&object, &name.into(), &callback.into_js_value())?;
        }
        Ok(object.into())
    })
}
fn json(value: &JsValue) -> String {
    js_sys::JSON::stringify(value).unwrap().as_string().unwrap()
}

#[wasm_bindgen_test]
fn timer_lifecycle_getters_errors_and_reentry_match_source() {
    let source = Closure::wrap(
        Box::new(source_factory) as Box<dyn Fn(JsValue, js_sys::Function) -> JsValue>
    );
    let expected = run_cases(source.as_ref().unchecked_ref()).unwrap();
    let rust = factory();
    let actual = run_cases(rust.as_ref().unchecked_ref()).unwrap();
    assert_eq!(js_sys::Array::from(&actual).length(), 38);
    assert_eq!(json(&actual), json(&expected));
    web_sys::console::log_1(&format!("CURSOR_BLINK_TRACES:{}", json(&actual)).into());
}

#[wasm_bindgen_test]
async fn actual_iframe_timers_and_animation_frames_match_source() {
    let source = Closure::wrap(
        Box::new(source_factory) as Box<dyn Fn(JsValue, js_sys::Function) -> JsValue>
    );
    let expected = native_driver(source.as_ref().unchecked_ref())
        .await
        .unwrap();
    let rust = factory();
    let actual = native_driver(rust.as_ref().unchecked_ref()).await.unwrap();
    assert_eq!(json(&actual), json(&expected));
    assert_eq!(json(&actual), "[[false,false],[true,false]]");
    web_sys::console::log_1(&format!("CURSOR_BLINK_NATIVE:{}", json(&actual)).into());
}

#[wasm_bindgen_test]
fn last_rust_owner_cancels_handles_and_expires_callbacks() {
    let rust = factory();
    let actual = drop_driver(rust.as_ref().unchecked_ref()).unwrap();
    assert_eq!(json(&actual), "[[\"clearTimeout\",1],[\"clearTimeout\",2]]");
}
