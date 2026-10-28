use super::{call, line};
use js_sys::{Array, Function, Reflect};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::buffer::constants::{attributes, content};

#[wasm_bindgen(
    inline_js = "export function throwingCell(log) { return { set content(v) { log.push('content'); }, set fg(v) { log.push('fg'); throw new RangeError('fg setter failure'); } }; }
export function mutatingCell(callback) { return { set content(v) { this.saved = v; callback(); }, get content() { return this.saved; } }; }
export function overriddenCell(mask) { return { set content(v) {}, get content() { return mask; } }; }"
)]
extern "C" {
    #[wasm_bindgen(js_name = throwingCell)]
    fn throwing_cell(log: &Array) -> JsValue;
    #[wasm_bindgen(js_name = mutatingCell)]
    fn mutating_cell(callback: &Function) -> JsValue;
    #[wasm_bindgen(js_name = overriddenCell)]
    fn overridden_cell(mask: u32) -> JsValue;
}

#[wasm_bindgen_test]
fn arbitrary_destination_fields_and_extended_mutations_remain_live() {
    let (_terminal, _normal, line) = line();
    let target: JsValue = js_sys::Object::new().into();
    assert_eq!(
        call(&line, "getCell", &[1.into(), target.clone()]).unwrap(),
        target
    );
    assert_eq!(
        Reflect::get(&target, &"combinedData".into())
            .unwrap()
            .as_string()
            .as_deref(),
        Some("e\u{301}")
    );
    let extended = Reflect::get(&target, &"extended".into()).unwrap();
    Reflect::set(
        &extended,
        &"underlineColor".into(),
        &JsValue::from(attributes::CM_RGB | 0xffaa),
    )
    .unwrap();
    let fresh = call(&line, "getCell", &[1.into()]).unwrap();
    assert_eq!(
        call(&fresh, "getUnderlineColor", &[]).unwrap().as_f64(),
        Some(65450.0)
    );
    let empty: JsValue = js_sys::Object::new().into();
    call(&line, "getCell", &[f64::NAN.into(), empty.clone()]).unwrap();
    assert!(Reflect::get(&empty, &"content".into())
        .unwrap()
        .is_undefined());
    assert!(Reflect::get(&empty, &"fg".into()).unwrap().is_undefined());
}

#[wasm_bindgen_test]
fn destination_setter_failures_preserve_partial_assignment_and_exception() {
    let (_terminal, _normal, line) = line();
    let log = Array::new();
    let target = throwing_cell(&log);
    let error = call(&line, "getCell", &[1.into(), target]).unwrap_err();
    assert_eq!(
        log.to_vec(),
        vec![JsValue::from_str("content"), JsValue::from_str("fg")]
    );
    assert_eq!(
        Reflect::get(&error, &"name".into())
            .unwrap()
            .as_string()
            .as_deref(),
        Some("RangeError")
    );
    assert_eq!(
        Reflect::get(&error, &"message".into())
            .unwrap()
            .as_string()
            .as_deref(),
        Some("fg setter failure")
    );
}

#[wasm_bindgen_test]
fn destination_callbacks_change_later_reads_and_choose_combined_data() {
    let (terminal, _normal, line) = line();
    let mutation = Rc::clone(&terminal);
    let callback = Closure::wrap(Box::new(move || {
        mutation
            .with_state(|state| {
                let buffer = state.active_buffer();
                let line = buffer.borrow().lines.get(0).unwrap();
                let mut cell = line.borrow().get_cell(0).unwrap();
                cell.attributes.fg = 456;
                line.borrow_mut().set_cell(0, &cell);
            })
            .unwrap();
    }) as Box<dyn FnMut()>);
    let target = mutating_cell(callback.as_ref().unchecked_ref());
    assert_eq!(
        call(&line, "getCell", &[0.into(), target.clone()]).unwrap(),
        target
    );
    assert_eq!(
        Reflect::get(&target, &"fg".into()).unwrap().as_f64(),
        Some(456.0)
    );
    let overridden = overridden_cell(content::IS_COMBINED_MASK);
    call(&line, "getCell", &[0.into(), overridden.clone()]).unwrap();
    assert!(Reflect::get(&overridden, &"combinedData".into())
        .unwrap()
        .is_undefined());
    let cell = call(&line, "getCell", &[0.into()]).unwrap();
    let error = call(&cell, "attributesEquals", &[js_sys::Object::new().into()]).unwrap_err();
    assert_eq!(
        Reflect::get(&error, &"name".into())
            .unwrap()
            .as_string()
            .as_deref(),
        Some("TypeError")
    );
}
