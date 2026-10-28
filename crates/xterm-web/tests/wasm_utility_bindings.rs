#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use js_sys::{JsString, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::{
    js_error::to_js_error, terminal::Terminal, utility_addons::bindings::ClipboardAddon,
};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn attach_constructor_matches_socket_first_order_and_primitive_option_boxing() {
    use xterm_web::utility_addons::bindings::AttachAddon;
    for options in [
        JsValue::UNDEFINED,
        JsValue::NULL,
        JsValue::FALSE,
        JsValue::TRUE,
        JsValue::from_f64(0.0),
        JsValue::from_f64(42.0),
        JsValue::from_str(""),
        JsValue::from_str("value"),
    ] {
        // Constructor-only host fixture. Real sockets are tested separately.
        let socket = Object::new();
        let addon = AttachAddon::new(socket.clone().unchecked_into(), options).unwrap();
        assert_eq!(
            Reflect::get(&socket, &"binaryType".into())
                .unwrap()
                .as_string()
                .unwrap(),
            "arraybuffer"
        );
        addon.dispose();
    }
    let socket = Object::new();
    let captured = socket.clone();
    let options = Object::new();
    let getter = Closure::wrap(Box::new(move || -> JsValue {
        assert_eq!(
            Reflect::get(&captured, &"binaryType".into())
                .unwrap()
                .as_string()
                .unwrap(),
            "arraybuffer"
        );
        JsValue::FALSE
    }) as Box<dyn FnMut() -> JsValue>);
    let descriptor = Object::new();
    Reflect::set(&descriptor, &"get".into(), getter.as_ref()).unwrap();
    Object::define_property(&options, &"bidirectional".into(), &descriptor);
    AttachAddon::new(socket.unchecked_into(), options.into())
        .unwrap()
        .dispose();
}

#[wasm_bindgen_test]
fn exported_clipboard_custom_provider_uses_real_terminal_osc_and_exact_replies() {
    let provider = Object::new();
    let saved = Rc::new(RefCell::new(String::new()));
    let store = saved.clone();
    let write = Closure::wrap(Box::new(move |_selection: JsString, text: JsString| {
        *store.borrow_mut() = text.as_string().unwrap();
    }) as Box<dyn FnMut(JsString, JsString)>);
    Reflect::set(&provider, &"writeText".into(), write.as_ref()).unwrap();
    let source = saved.clone();
    let read = Closure::wrap(Box::new(move |_selection: JsString| -> JsValue {
        JsString::from(source.borrow().as_str()).into()
    }) as Box<dyn FnMut(JsString) -> JsValue>);
    Reflect::set(&provider, &"readText".into(), read.as_ref()).unwrap();
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let addon = ClipboardAddon::new(JsValue::UNDEFINED, provider.into());
    addon.activate(&terminal).unwrap();
    let replies = Rc::new(RefCell::new(Vec::new()));
    let capture = replies.clone();
    terminal
        .shared_native()
        .on_data()
        .subscribe(move |data| capture.borrow_mut().push(data.clone()));
    for selection in ["c", "p", ""] {
        terminal
            .shared_native()
            .write(&format!(
                "\x1b]52;{selection};4oKsbWzDpMO8dMOf\x07\x1b]52;{selection};?\x1b\\"
            ))
            .unwrap();
        assert_eq!(*saved.borrow(), "€mläütß");
        assert_eq!(
            String::from_utf16(replies.borrow().last().unwrap()).unwrap(),
            format!("\x1b]52;{selection};4oKsbWzDpMO8dMOf\x07")
        );
    }
    terminal.shared_native().write("\x1b]52;c;!\x07").unwrap();
    assert_eq!(*saved.borrow(), "");
    addon.dispose();
    let count = replies.borrow().len();
    terminal.shared_native().write("\x1b]52;c;?\x07").unwrap();
    assert_eq!(replies.borrow().len(), count);
    terminal.dispose().unwrap();
}

#[wasm_bindgen_test]
fn exported_clipboard_preserves_original_javascript_provider_exception() {
    let provider = Object::new();
    let original = Object::new();
    let thrown = original.clone();
    let read = Closure::wrap(
        Box::new(move |_selection: JsString| -> Result<JsValue, JsValue> {
            Err(thrown.clone().into())
        }) as Box<dyn FnMut(JsString) -> Result<JsValue, JsValue>>,
    );
    Reflect::set(&provider, &"readText".into(), read.as_ref()).unwrap();
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let addon = ClipboardAddon::new(JsValue::UNDEFINED, provider.into());
    addon.activate(&terminal).unwrap();
    let error = terminal
        .shared_native()
        .write("\x1b]52;c;?\x07must-not-print")
        .unwrap_err();
    assert!(Object::is(&to_js_error(&error), original.as_ref()));
    addon.dispose();
    terminal.dispose().unwrap();
}
