#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = "export function domProbe(terminal, log) {return {activate(t) {log.push(t===terminal);},dispose() {log.push(terminal.element.isConnected);}};}")]
extern "C" { #[wasm_bindgen(js_name = domProbe)] fn dom_probe(terminal: &JsValue, log: &Array) -> JsValue; }
fn dispose(object: &JsValue) -> Result<JsValue, JsValue> { Reflect::get(object, &"dispose".into())?.dyn_into::<Function>()?.call0(object) }

#[wasm_bindgen_test]
fn actual_terminal_identity_and_core_dom_cleanup_precede_addon_disposal() {
    let document = web_sys::window().unwrap().document().unwrap();
    let style = document.create_element("style").unwrap();
    style.set_text_content(Some(include_str!("../assets/xterm.css")));
    document.body().unwrap().append_child(&style).unwrap();
    let container = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&container).unwrap();
    let terminal: JsValue = xterm_web::terminal::Terminal::new(js_sys::Object::new().into())
        .unwrap()
        .into();
    Reflect::get(&terminal, &"open".into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap()
        .call1(&terminal, &container)
        .unwrap();
    let log = Array::new();
    let addon = dom_probe(&terminal, &log);
    Reflect::get(&terminal, &"_loadAddon".into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap()
        .call2(&terminal, &terminal, &addon)
        .unwrap();
    assert_eq!(log.get(0).as_bool(), Some(true));
    dispose(&terminal).unwrap();
    assert_eq!(log.get(1).as_bool(), Some(false));
    container.remove();
    style.remove();
}
