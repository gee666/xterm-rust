#![cfg(target_arch = "wasm32")]
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::scroll::fast_dom_node::FastDomNode;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "
const records = new WeakMap();
export function instrumentFast(node) {
  const style=node.style; const writes=[];
  const proxy=new Proxy(style,{get(target,key){const value=Reflect.get(target,key,target);return typeof value==='function'?value.bind(target):value;},set(target,key,value){writes.push([key,value]);return Reflect.set(target,key,value,target);}});
  records.set(node,{style,proxy,writes});
  Object.defineProperty(node,'style',{configurable:true,get(){return proxy;}});
}
export function fastWrites(node){return records.get(node).writes;}
export function fastCoercible(log){return {[Symbol.toPrimitive](hint){log.push(hint);return '4px';}};}
export function fastReenter(node,callback){let once=true;const state=records.get(node);Object.defineProperty(node,'style',{configurable:true,get(){if(once){once=false;callback();}return state.proxy;}});}
export function restoreFast(node){delete node.style;}
")]
extern "C" {
    #[wasm_bindgen(js_name = instrumentFast)]
    fn instrument(node: &HtmlElement);
    #[wasm_bindgen(js_name = fastWrites)]
    fn writes(node: &HtmlElement) -> js_sys::Array;
    #[wasm_bindgen(js_name = fastCoercible)]
    fn coercible(log: &js_sys::Array) -> JsValue;
    #[wasm_bindgen(js_name = fastReenter)]
    fn reenter(node: &HtmlElement, callback: &js_sys::Function);
    #[wasm_bindgen(js_name = restoreFast)]
    fn restore(node: &HtmlElement);
}
fn element() -> HtmlElement {
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .create_element("div")
        .unwrap()
        .dyn_into()
        .unwrap()
}

#[wasm_bindgen_test]
fn style_cache_skips_initial_empty_and_duplicate_values() {
    let node = element();
    node.style().set_property("width", "17px").unwrap();
    instrument(&node);
    let fast = FastDomNode::new(node.clone());
    fast.set_width("").unwrap();
    assert_eq!(writes(&node).length(), 0);
    fast.set_width(&2.5).unwrap();
    fast.set_width("2.5px").unwrap();
    assert_eq!(writes(&node).length(), 1);
    assert_eq!(node.style().get_property_value("width").unwrap(), "2.5px");
    restore(&node);
}

#[wasm_bindgen_test]
fn raw_object_gets_string_hint_once_and_retains_cache_identity() {
    let node = element();
    instrument(&node);
    let fast = FastDomNode::new(node.clone());
    let log = js_sys::Array::new();
    let value = coercible(&log);
    fast.set_width(&value).unwrap();
    fast.set_width(&value).unwrap();
    assert_eq!(log.length(), 1);
    assert_eq!(log.get(0).as_string().unwrap(), "string");
    assert_eq!(writes(&node).length(), 1);
    assert_eq!(node.style().get_property_value("width").unwrap(), "4px");
    restore(&node);
}

#[wasm_bindgen_test]
fn style_getter_reentry_reads_the_updated_cache_rhs() {
    let node = element();
    instrument(&node);
    let fast = FastDomNode::new(node.clone());
    let nested = fast.clone();
    let callback: Closure<dyn Fn() -> Result<(), JsValue>> =
        Closure::wrap(Box::new(move || nested.set_width("9px")));
    reenter(&node, callback.as_ref().unchecked_ref());
    fast.set_width("4px").unwrap();
    assert_eq!(node.style().get_property_value("width").unwrap(), "9px");
    let calls = writes(&node);
    assert_eq!(calls.length(), 2);
    for entry in calls.iter() {
        assert_eq!(
            js_sys::Array::from(&entry).get(1).as_string().unwrap(),
            "9px"
        );
    }
    restore(&node);
}

#[wasm_bindgen_test]
fn containment_and_layer_hint_have_source_initial_cache_values() {
    let node = element();
    node.style().set_property("contain", "paint").unwrap();
    node.style().set_property("transform", "scale(2)").unwrap();
    let fast = FastDomNode::new(node.clone());
    fast.set_contain("none").unwrap();
    fast.set_layer_hinting(false).unwrap();
    assert_eq!(node.style().get_property_value("contain").unwrap(), "paint");
    assert_eq!(
        node.style().get_property_value("transform").unwrap(),
        "scale(2)"
    );
    fast.set_contain("strict").unwrap();
    fast.set_layer_hinting(true).unwrap();
    assert_eq!(
        node.style().get_property_value("contain").unwrap(),
        "strict"
    );
    assert_eq!(
        node.style().get_property_value("transform").unwrap(),
        "translate3d(0px, 0px, 0px)"
    );
}

#[wasm_bindgen_test]
fn toggle_refreshes_class_cache_and_attributes_are_not_deduplicated() {
    let node = element();
    let fast = FastDomNode::new(node.clone());
    fast.set_class_name("first").unwrap();
    fast.toggle_class_name(&JsValue::from_str("second"), Some(true))
        .unwrap();
    fast.set_class_name("first second").unwrap();
    assert_eq!(node.class_name(), "first second");
    fast.toggle_class_name(&JsValue::from_str("second"), None)
        .unwrap();
    assert_eq!(node.class_name(), "first");
    fast.set_attribute("aria-label", "value").unwrap();
    assert_eq!(node.get_attribute("aria-label").as_deref(), Some("value"));
}
