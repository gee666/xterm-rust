#![cfg(target_arch = "wasm32")]
// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
//! JS-retained listeners outlive failed registration/removal Rust handles.
use js_sys::{Function, Reflect};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_addons::attach::{AttachSocket, SocketEvent};
use xterm_web::utility_addons::socket::BrowserSocket;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn js_retained_callbacks_survive_failed_registration_and_dropped_removal_handles() {
    for registration_throws in [false, true] {
        let value = Function::new_with_args(
            "fail",
            r#"
          return { addEventListener(kind, callback) {
            this.saved = callback;
            if (fail) throw 'registration';
          }, removeEventListener() { throw 'removal'; } };
        "#,
        )
        .call1(&JsValue::UNDEFINED, &registration_throws.into())
        .unwrap();
        let socket = BrowserSocket(value.clone().unchecked_into());
        let called = Rc::new(Cell::new(0));
        let count = called.clone();
        let result = socket.listen(
            SocketEvent::Close,
            Rc::new(move |_| {
                count.set(count.get() + 1);
                Ok(())
            }),
        );
        if registration_throws {
            assert!(result.is_err());
        } else {
            let listener = result.unwrap();
            assert!(listener.remove().is_err());
            assert!(listener.remove().is_err());
            drop(listener);
        }
        drop(socket);
        let callback = Reflect::get(&value, &"saved".into())
            .unwrap()
            .unchecked_into::<Function>();
        callback.call1(&value, &js_sys::Object::new()).unwrap();
        callback.call1(&value, &js_sys::Object::new()).unwrap();
        assert_eq!(called.get(), 2);
        Reflect::delete_property(&value.unchecked_into::<js_sys::Object>(), &"saved".into())
            .unwrap();
    }
}
