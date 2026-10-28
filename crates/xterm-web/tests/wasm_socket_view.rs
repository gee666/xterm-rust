#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
use js_sys::{ArrayBuffer, Function, Promise, Uint8Array};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::{Event, MessageEvent, WebSocket};
use xterm_web::{terminal::Terminal, utility_addons::bindings::AttachAddon};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen_test(async)]
async fn later_message_listener_mutation_is_visible_when_rust_queue_parses() {
    let Some(url) = option_env!("UTILITY_SOCKET_URL") else {
        panic!("run the utility WebSocket browser wrapper");
    };
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let socket = WebSocket::new(url).unwrap();
    let addon = AttachAddon::new(socket.clone(), JsValue::UNDEFINED).unwrap();
    addon.activate(&terminal).unwrap();
    let resolved = Rc::new(RefCell::new(None::<Function>));
    let install = resolved.clone();
    let done = Promise::new(&mut move |resolve, _| *install.borrow_mut() = Some(resolve));
    let native = terminal.shared_native();
    let weak = Rc::downgrade(&native);
    let parsed = native.on_write_parsed().subscribe(move |_| {
        let native = weak.upgrade().unwrap();
        let text = native
            .with_state(|state| {
                let active = state.active_buffer();
                let buffer = active.borrow();
                let line = buffer.lines.get(buffer.ybase + buffer.y).unwrap();
                let text = line.borrow().translate_to_string(true, None, None, None);
                text
            })
            .unwrap();
        if String::from_utf16(&text).unwrap() == "fooboo" {
            if let Some(resolve) = resolved.borrow_mut().take() {
                resolve.call0(&JsValue::UNDEFINED).unwrap();
            }
        }
    });
    let mutated = Rc::new(Cell::new(false));
    let changed = mutated.clone();
    let listener = Closure::wrap(Box::new(move |event: Event| {
        let event = event.dyn_into::<MessageEvent>().unwrap();
        let data = event.data();
        if data.is_instance_of::<ArrayBuffer>() {
            Uint8Array::new(&data).set_index(0, b'b');
            changed.set(true);
        }
    }) as Box<dyn FnMut(Event)>);
    socket
        .add_event_listener_with_callback("message", listener.as_ref().unchecked_ref())
        .unwrap();
    JsFuture::from(done).await.unwrap();
    assert!(mutated.get());
    socket
        .remove_event_listener_with_callback("message", listener.as_ref().unchecked_ref())
        .unwrap();
    parsed.dispose();
    addon.dispose();
    socket.close().unwrap();
    terminal.dispose().unwrap();
}
