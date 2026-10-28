#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Object, Reflect};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_core::{event::Emitter, lifecycle::Disposable};
use xterm_wasm_support::events::{callable, from_callback, from_js_callback};
wasm_bindgen_test_configure!(run_in_browser);
fn dispose(token: &JsValue) -> Function {
    Reflect::get(token, &"dispose".into())
        .unwrap()
        .dyn_into()
        .unwrap()
}
fn listener(state: &JsValue, body: &str) -> Function {
    Function::new_with_args("state", &format!("return function(payload){{{body}}};"))
        .call1(&JsValue::UNDEFINED, state)
        .unwrap()
        .dyn_into()
        .unwrap()
}

#[wasm_bindgen_test]
fn supplied_disposal_function_keeps_identity_receiver_and_plain_properties() {
    let callback = Function::new_no_args("'use strict';return this;");
    let token = from_js_callback(&callback);
    let extracted = dispose(&token);
    assert_eq!(extracted, callback);
    assert_eq!(extracted.call0(&token).unwrap(), token);
    assert!(extracted.call0(&JsValue::UNDEFINED).unwrap().is_undefined());
    Function::new_with_args("token","const d=Object.getOwnPropertyDescriptor(token,'dispose');if(Object.keys(token).join(',')!=='dispose'||!d.writable||!d.enumerable||!d.configurable)throw Error('not a plain disposal token');")
        .call1(&JsValue::UNDEFINED,&token).unwrap();
}

#[wasm_bindgen_test]
fn native_disposal_is_reentrant_and_preserves_return_and_throw_values() {
    let slot = Rc::new(RefCell::new(None::<Function>));
    let weak = Rc::downgrade(&slot);
    let calls = Rc::new(Cell::new(0));
    let output = calls.clone();
    let marker: JsValue = Object::new().into();
    let returned = marker.clone();
    let token = from_callback(move || {
        let count = output.get() + 1;
        output.set(count);
        if count == 1 {
            let function = weak.upgrade().unwrap().borrow().as_ref().unwrap().clone();
            function.call0(&JsValue::UNDEFINED)?;
        }
        Ok(returned.clone())
    });
    let function = dispose(&token);
    *slot.borrow_mut() = Some(function.clone());
    assert_eq!(function.call0(&JsValue::UNDEFINED).unwrap(), marker);
    assert_eq!(calls.get(), 2);
    assert_eq!(function.call0(&JsValue::UNDEFINED).unwrap(), marker);
    assert_eq!(calls.get(), 3);
    let thrown = marker.clone();
    let failed = from_callback(move || Err(thrown.clone()));
    assert_eq!(
        dispose(&failed).call0(&JsValue::UNDEFINED).unwrap_err(),
        marker
    );
}

#[wasm_bindgen_test]
fn callable_shares_one_payload_per_fire_and_extracted_dispose_unregisters() {
    let emitter = Emitter::new();
    let conversions = Rc::new(Cell::new(0));
    let count = conversions.clone();
    let event = callable(emitter.event(), move |value: &u32| {
        count.set(count.get() + 1);
        let object = Object::new();
        Reflect::set(&object, &"value".into(), &JsValue::from(*value))?;
        Ok(object.into())
    });
    let state: JsValue = Object::new().into();
    let container = Array::new();
    let first = listener(&state, "state.first=payload;payload.changed=true;");
    let second=listener(&state,"state.same=payload===state.first;state.changed=payload.changed===true;state.current=payload;");
    let token = event
        .call3(&JsValue::UNDEFINED, &first, &JsValue::UNDEFINED, &container)
        .unwrap();
    event
        .call3(
            &JsValue::UNDEFINED,
            &second,
            &JsValue::UNDEFINED,
            &container,
        )
        .unwrap();
    assert_eq!(container.length(), 2);
    emitter.fire(&1).unwrap();
    assert_eq!(conversions.get(), 1);
    assert_eq!(Reflect::get(&state, &"same".into()).unwrap(), JsValue::TRUE);
    assert_eq!(
        Reflect::get(&state, &"changed".into()).unwrap(),
        JsValue::TRUE
    );
    dispose(&token).call0(&JsValue::UNDEFINED).unwrap();
    emitter.fire(&2).unwrap();
    assert_eq!(conversions.get(), 2);
    assert_eq!(
        Reflect::get(&state, &"same".into()).unwrap(),
        JsValue::FALSE
    );
    assert_eq!(
        Reflect::get(&state, &"changed".into()).unwrap(),
        JsValue::FALSE
    );
}

#[wasm_bindgen_test]
fn callable_passes_this_args_and_the_custom_collection_receiver() {
    let emitter = Emitter::new();
    let event = callable(emitter.event(), |value: &u32| Ok(JsValue::from(*value)));
    let state: JsValue = Object::new().into();
    let callback = Function::new_with_args("value", "'use strict';this.last=value;");
    let container = Function::new_with_args(
        "state",
        "return {add(token){state.token=token;state.receiver=this;}};",
    )
    .call1(&JsValue::UNDEFINED, &state)
    .unwrap();
    let token = event
        .call3(&JsValue::UNDEFINED, &callback, &state, &container)
        .unwrap();
    assert_eq!(Reflect::get(&state, &"token".into()).unwrap(), token);
    assert_eq!(Reflect::get(&state, &"receiver".into()).unwrap(), container);
    emitter.fire(&7).unwrap();
    assert_eq!(
        Reflect::get(&state, &"last".into()).unwrap(),
        JsValue::from(7)
    );
    dispose(&token).call0(&JsValue::UNDEFINED).unwrap();
}

#[wasm_bindgen_test]
fn closed_subscriptions_get_fresh_noops_and_do_not_collect() {
    let emitter = Emitter::<()>::new();
    let event = callable(emitter.event(), |_| Ok(JsValue::UNDEFINED));
    emitter.dispose();
    let container = Function::new_no_args(
        "return {add(){throw Error('must not collect closed registration')}};",
    )
    .call0(&JsValue::UNDEFINED)
    .unwrap();
    let callback = Function::new_no_args("throw Error('closed listener ran')");
    let first = event
        .call3(
            &JsValue::UNDEFINED,
            &callback,
            &JsValue::UNDEFINED,
            &container,
        )
        .unwrap();
    let second = event
        .call3(
            &JsValue::UNDEFINED,
            &callback,
            &JsValue::UNDEFINED,
            &container,
        )
        .unwrap();
    assert_ne!(first, second);
    assert_ne!(dispose(&first), dispose(&second));
    dispose(&first).call0(&JsValue::UNDEFINED).unwrap();
    emitter.fire(&()).unwrap();
}

#[wasm_bindgen_test]
fn collection_error_preserves_registered_listener_and_original_thrown_value() {
    let emitter = Emitter::new();
    let event = callable(emitter.event(), |value: &u32| Ok(JsValue::from(*value)));
    let state: JsValue = Object::new().into();
    let callback = listener(&state, "state.value=payload;");
    let container = Function::new_no_args(
        "const marker={tag:'collection'};return {marker,add(){throw marker;}};",
    )
    .call0(&JsValue::UNDEFINED)
    .unwrap();
    let error = event
        .call3(
            &JsValue::UNDEFINED,
            &callback,
            &JsValue::UNDEFINED,
            &container,
        )
        .unwrap_err();
    assert_eq!(error, Reflect::get(&container, &"marker".into()).unwrap());
    emitter.fire(&9).unwrap();
    assert_eq!(
        Reflect::get(&state, &"value".into()).unwrap(),
        JsValue::from(9)
    );
}
