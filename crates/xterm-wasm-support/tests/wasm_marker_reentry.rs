#![cfg(target_arch = "wasm32")]
use std::rc::Rc;
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_core::headless::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn marker_callback_can_cross_js_and_reset_the_rust_terminal() {
    let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
    terminal.write("abcde12345").unwrap();
    let marker = terminal.register_marker(0).unwrap();
    let weak = Rc::downgrade(&terminal);
    let reset = Closure::<dyn FnMut()>::new(move || weak.upgrade().unwrap().reset().unwrap());
    let function: js_sys::Function = reset.as_ref().unchecked_ref::<js_sys::Function>().clone();
    marker.on_dispose().subscribe(move |_| {
        function.call0(&JsValue::UNDEFINED).unwrap();
    });
    terminal.write("\x1b[2Jafter").unwrap();
    let active = terminal.with_state(|state| state.active_buffer()).unwrap();
    let row = active.borrow().lines.get(0).unwrap();
    assert_eq!(
        row.borrow().translate_to_string(true, None, None, None),
        "after".encode_utf16().collect::<Vec<_>>()
    );
    assert!(marker.is_disposed());
    drop(reset);
}
