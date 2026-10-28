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
    let addon = ClipboardAddon::new(JsValue::UNDEFINED, provider.into()).unwrap();
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
    let addon = ClipboardAddon::new(JsValue::UNDEFINED, provider.into()).unwrap();
    addon.activate(&terminal).unwrap();
    let error = terminal
        .shared_native()
        .write("\x1b]52;c;?\x07must-not-print")
        .unwrap_err();
    assert!(Object::is(&to_js_error(&error), original.as_ref()));
    addon.dispose();
    terminal.dispose().unwrap();
}
