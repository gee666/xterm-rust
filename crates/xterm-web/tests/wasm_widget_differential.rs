#![cfg(target_arch = "wasm32")]
//! Shared source/Rust Widget assertions exercise actual disposable DOM listeners.
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_core::lifecycle::{Disposable, DisposableStore};
use xterm_web::scroll::{
    iframe::IframeCache,
    mouse_event::StandardMouseEvent,
    widget::{self, MouseListener},
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/fixtures/widget-cases.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runWidgetCase)]
    fn run_case(name: &str, factory: &js_sys::Function) -> Result<u32, JsValue>;
}
fn mouse_value(event: StandardMouseEvent) -> Result<JsValue, JsValue> {
    let object = js_sys::Object::new();
    for (name, value) in [
        ("browserEvent", event.browser_event.clone().into()),
        ("leftButton", event.left_button.into()),
        ("middleButton", event.middle_button.into()),
        ("rightButton", event.right_button.into()),
        ("target", event.target.clone()),
        ("buttons", event.buttons.clone()),
        ("detail", event.detail.clone()),
        ("posx", event.posx.into()),
        ("posy", event.posy.into()),
        ("ctrlKey", event.ctrl_key.clone()),
        ("shiftKey", event.shift_key.clone()),
        ("timestamp", event.timestamp.clone()),
    ] {
        js_sys::Reflect::set(&object, &name.into(), &value)?;
    }
    for name in ["preventDefault", "stopPropagation"] {
        let event = event.clone();
        let callback = Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || {
            if name == "preventDefault" {
                event.prevent_default()
            } else {
                event.stop_propagation()
            }
        });
        js_sys::Reflect::set(&object, &name.into(), &callback.into_js_value())?;
    }
    Ok(object.into())
}
fn register(
    node: HtmlElement,
    kind: String,
    callback: js_sys::Function,
) -> Result<JsValue, JsValue> {
    let store = Rc::new(DisposableStore::new());
    let listener: MouseListener = Rc::new(move |event| {
        callback.call1(&JsValue::UNDEFINED, &mouse_value(event)?)?;
        Ok(())
    });
    let window = web_sys::window().unwrap();
    let cache = Rc::new(IframeCache::new());
    match kind.as_str() {
        "click" => widget::on_click(&store, &node, &window, cache, listener)?,
        "mouseover" => widget::on_mouseover(&store, &node, &window, cache, listener)?,
        "mouseleave" => widget::on_mouseleave(&store, &node, &window, cache, listener)?,
        _ => panic!("only source Widget listener kinds are registered"),
    }
    let dispose = Closure::<dyn Fn()>::new(move || store.dispose());
    Ok(dispose.into_js_value())
}
fn check(name: &str) {
    let factory = Closure::<
        dyn Fn(HtmlElement, String, js_sys::Function) -> Result<JsValue, JsValue>,
    >::new(register);
    assert!(run_case(name, factory.as_ref().unchecked_ref()).unwrap() > 0);
}
#[wasm_bindgen_test]
fn kinds_and_cleanup() {
    check("kinds_and_cleanup");
}
#[wasm_bindgen_test]
fn recursive_disposal() {
    check("recursive_disposal");
}
#[wasm_bindgen_test]
fn adopted_window() {
    check("adopted_window");
}
#[wasm_bindgen_test]
fn callback_error() {
    check("callback_error");
}
