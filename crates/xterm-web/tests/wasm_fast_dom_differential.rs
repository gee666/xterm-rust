#![cfg(target_arch = "wasm32")]
//! Shared browser assertions also run against unchanged upstream FastDomNode.
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::{scroll::fast_dom_node::FastDomNode, terminal::options::from_js};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/fixtures/fast-dom-cases.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runFastCase)]
    fn run_case(name: &str, factory: &js_sys::Function) -> Result<u32, JsValue>;
}

fn adapter(node: HtmlElement) -> Result<JsValue, JsValue> {
    let fast = FastDomNode::new(node);
    let object = js_sys::Object::new();
    for name in [
        "setWidth",
        "setHeight",
        "setTop",
        "setLeft",
        "setBottom",
        "setRight",
        "setPosition",
        "setContain",
        "setClassName",
        "toggleClassName",
        "setLayerHinting",
        "setAttribute",
    ] {
        let fast = fast.clone();
        let method = Closure::<dyn Fn(JsValue, JsValue) -> Result<(), JsValue>>::new(
            move |a: JsValue, b: JsValue| match name {
                "setWidth" => fast.set_width(&a),
                "setHeight" => fast.set_height(&a),
                "setTop" => fast.set_top(&a),
                "setLeft" => fast.set_left(&a),
                "setBottom" => fast.set_bottom(&a),
                "setRight" => fast.set_right(&a),
                "setPosition" => fast.set_position(&a),
                "setContain" => fast.set_contain(&a),
                "setClassName" => fast.set_class_name(&a),
                "toggleClassName" => fast.toggle_class_name_raw(&from_js(&a)?, &from_js(&b)?),
                "setLayerHinting" => fast.set_layer_hinting(a.is_truthy()),
                "setAttribute" => fast.set_attribute_raw(&from_js(&a)?, &from_js(&b)?),
                _ => unreachable!("method names are fixed above"),
            },
        );
        js_sys::Reflect::set(&object, &name.into(), &method.into_js_value())?;
    }
    Ok(object.into())
}
fn check(name: &str) {
    let factory = Closure::<dyn Fn(HtmlElement) -> Result<JsValue, JsValue>>::new(adapter);
    assert!(run_case(name, factory.as_ref().unchecked_ref()).unwrap() > 0);
}
#[wasm_bindgen_test]
fn dimensions() {
    check("dimensions");
}
#[wasm_bindgen_test]
fn failed_assignment() {
    check("failed_assignment");
}
#[wasm_bindgen_test]
fn failed_conversion() {
    check("failed_conversion");
}
#[wasm_bindgen_test]
fn class_reentry() {
    check("class_reentry");
}
#[wasm_bindgen_test]
fn class_toggle() {
    check("class_toggle");
}
#[wasm_bindgen_test]
fn attributes() {
    check("attributes");
}
#[wasm_bindgen_test]
fn raw_utf16() {
    check("raw_utf16");
}
#[wasm_bindgen_test]
fn layer_hint() {
    check("layer_hint");
}
