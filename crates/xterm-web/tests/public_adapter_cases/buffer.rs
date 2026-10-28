#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, Reflect};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::headless::Terminal;
use xterm_web::terminal::buffer_api::WebBufferNamespace;

#[wasm_bindgen(
    inline_js = "export function capture(log) {return value => log.push(value);}\nexport function captureReceiver(log, expected) {return function(value) {log.push([this===expected,value]);};}"
)]
extern "C" {
    fn capture(log: &Array) -> Function;
    #[wasm_bindgen(js_name = captureReceiver)]
    fn capture_receiver(log: &Array, expected: &JsValue) -> Function;
}
fn call(object: &JsValue, name: &str, args: &[JsValue]) -> JsValue {
    let method = Reflect::get(object, &name.into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap();
    let arguments: Array = args.iter().cloned().collect();
    method.apply(object, &arguments).unwrap()
}
fn property(object: &JsValue, name: &str) -> JsValue {
    Reflect::get(object, &name.into()).unwrap()
}

#[wasm_bindgen_test]
fn javascript_buffer_and_line_identities_are_live_and_cell_targets_are_reused() {
    let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
    let namespace = WebBufferNamespace::new(terminal.clone()).unwrap();
    let normal = namespace.normal();
    assert_eq!(normal, namespace.normal());
    assert_eq!(normal, namespace.active().unwrap());
    terminal.write("abc").unwrap();
    assert_eq!(property(&normal, "cursorX").as_f64(), Some(3.0));
    let line = call(&normal, "getLine", &[0.into()]);
    assert_eq!(line, call(&normal, "getLine", &[0.into()]));
    let cell = call(&line, "getCell", &[0.into()]);
    assert_eq!(
        call(&cell, "getChars", &[]).as_string().as_deref(),
        Some("a")
    );
    let target = call(&normal, "getNullCell", &[]);
    let expected =
        js_sys::JSON::parse(include_str!("../fixtures/public-cell-methods.json")).unwrap();
    for entry in js_sys::Object::entries(&expected.into()).iter() {
        let entry = Array::from(&entry);
        let name = entry.get(0).as_string().unwrap();
        let arguments = if name == "attributesEquals" {
            vec![target.clone()]
        } else {
            Vec::new()
        };
        assert_eq!(
            call(&target, &name, &arguments),
            entry.get(1),
            "method {name}"
        );
    }
    let result = call(&line, "getCell", &[1.into(), target.clone()]);
    assert_eq!(result, target);
    assert_eq!(
        call(&target, "getChars", &[]).as_string().as_deref(),
        Some("b")
    );
    assert_eq!(
        call(&line, "translateToString", &[true.into()])
            .as_string()
            .as_deref(),
        Some("abc")
    );
    assert!(call(&line, "getCell", &[(-1).into()]).is_undefined());
}

#[wasm_bindgen_test]
fn namespace_event_function_keeps_identity_and_does_not_require_a_receiver() {
    let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
    let namespace = WebBufferNamespace::new(terminal.clone()).unwrap();
    let event: JsValue = namespace.on_buffer_change_function().into();
    assert_eq!(event, JsValue::from(namespace.on_buffer_change_function()));
    let log = Array::new();
    let receiver: JsValue = js_sys::Object::new().into();
    let collector = Array::new();
    let event = event.dyn_into::<Function>().unwrap();
    let subscription = event
        .call3(
            &JsValue::UNDEFINED,
            &capture_receiver(&log, &receiver),
            &receiver,
            &collector,
        )
        .unwrap();
    assert_eq!(collector.get(0), subscription);
    terminal.write("\x1b[?1049h").unwrap();
    let delivered = Array::from(&log.get(0));
    assert_eq!(delivered.get(0).as_bool(), Some(true));
    assert_eq!(delivered.get(1), namespace.alternate());
    let cleanup = Reflect::get(&subscription, &"dispose".into())
        .unwrap()
        .dyn_into::<Function>()
        .unwrap();
    cleanup.call0(&JsValue::UNDEFINED).unwrap();
    terminal.write("\x1b[?1049l").unwrap();
    assert_eq!(log.length(), 1);
    terminal.dispose().unwrap();
    let ignored = Array::new();
    let first = event
        .call3(
            &JsValue::UNDEFINED,
            &capture(&log),
            &JsValue::UNDEFINED,
            &ignored,
        )
        .unwrap();
    let second = event.call1(&JsValue::UNDEFINED, &capture(&log)).unwrap();
    assert_ne!(first, second);
    assert_eq!(ignored.length(), 0);
}

#[wasm_bindgen_test]
fn namespace_callbacks_reuse_public_objects_across_switch_reset_and_disposal() {
    let terminal = Rc::new(Terminal::new(5, 3, 10).unwrap());
    let namespace = WebBufferNamespace::new(terminal.clone()).unwrap();
    let normal = namespace.normal();
    let alternate = namespace.alternate();
    let log = Array::new();
    let subscription = namespace
        .on_buffer_change_function()
        .call1(&JsValue::UNDEFINED, &capture(&log))
        .unwrap();
    terminal.write("\x1b[?1049h").unwrap();
    assert_eq!(namespace.active().unwrap(), alternate);
    terminal.write("\x1b[?1049l").unwrap();
    terminal.reset().unwrap();
    assert_eq!(namespace.active().unwrap(), normal);
    assert_eq!(log.length(), 3);
    assert_eq!(log.get(0), alternate);
    assert_eq!(log.get(1), normal);
    assert_eq!(log.get(2), normal);
    call(&subscription, "dispose", &[]);
    call(&subscription, "dispose", &[]);
    terminal.write("\x1b[?1049h").unwrap();
    assert_eq!(log.length(), 3);
}
