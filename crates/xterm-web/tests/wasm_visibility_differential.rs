#![cfg(target_arch = "wasm32")]
//! Source/Rust assertions share deterministic host timers, errors and DOM writes.
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::{
    scroll::{
        browser_visibility::BrowserVisibility, fast_dom_node::FastDomNode, ScrollbarVisibility,
    },
    terminal::options::from_js,
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/fixtures/visibility-cases.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runVisibilityCase)]
    fn run_case(name: &str, factory: &js_sys::Function) -> Result<u32, JsValue>;
}
fn adapter(mode: JsValue, node: HtmlElement) -> Result<JsValue, JsValue> {
    let controller = BrowserVisibility::new(
        web_sys::window().unwrap(),
        ScrollbarVisibility::from_value(from_js(&mode)?),
        "shown",
        "hidden",
    )?;
    controller.set_dom_node(FastDomNode::new(node))?;
    let object = js_sys::Object::new();
    for name in [
        "setVisibility",
        "setDomNode",
        "setIsNeeded",
        "setShouldBeVisible",
        "ensureVisibility",
        "dispose",
    ] {
        let controller = controller.clone();
        let method = Closure::<dyn Fn(JsValue) -> Result<(), JsValue>>::new(
            move |value: JsValue| match name {
                "setVisibility" => {
                    controller.set_visibility(ScrollbarVisibility::from_value(from_js(&value)?))
                }
                "setDomNode" => controller.set_dom_node(FastDomNode::new(value.dyn_into()?)),
                "setIsNeeded" => controller.set_is_needed(value.is_truthy()),
                "setShouldBeVisible" => controller.set_should_be_visible(value.is_truthy()),
                "ensureVisibility" => controller.ensure_visibility(),
                "dispose" => controller.dispose(),
                _ => unreachable!("methods fixed above"),
            },
        );
        js_sys::Reflect::set(&object, &name.into(), &method.into_js_value())?;
    }
    Ok(object.into())
}
fn check(name: &str) {
    let factory = Closure::<dyn Fn(JsValue, HtmlElement) -> Result<JsValue, JsValue>>::new(adapter);
    assert!(run_case(name, factory.as_ref().unchecked_ref()).unwrap() > 0);
}
#[wasm_bindgen_test]
fn modes_and_timing() {
    check("modes_and_timing");
}
#[wasm_bindgen_test]
fn replacement() {
    check("replacement");
}
#[wasm_bindgen_test]
fn dispose() {
    check("dispose");
}
#[wasm_bindgen_test]
fn host_errors() {
    check("host_errors");
}
#[wasm_bindgen_test]
fn class_errors() {
    check("class_errors");
}
