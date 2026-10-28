// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use crate::accessibility_support::{text, units, Fixture};
use js_sys::{Function, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;
use xterm_web::accessibility::AccessibilityEvent as Event;
const OVERFLOW_LINES: usize = 21;

#[wasm_bindgen_test]
fn echo_tab_linefeed_and_blur_preserve_pending_output() {
    let f = Fixture::new();
    f.host.emit(Event::Character(units("out"))).unwrap();
    f.host.emit(Event::Key(units("x"))).unwrap();
    f.host.emit(Event::Character(units("x"))).unwrap();
    f.host.emit(Event::Tab(2)).unwrap();
    f.host.emit(Event::LineFeed).unwrap();
    assert!(text(&f.live()).is_empty());
    f.render();
    assert_eq!(text(&f.live()), units("out  \n"));
    f.host.emit(Event::Blur).unwrap();
    assert!(text(&f.live()).is_empty());
    f.host.emit(Event::Key(vec![13])).unwrap();
    f.host.emit(Event::Character(vec![13])).unwrap();
    f.render();
    assert_eq!(text(&f.live()), [13]);
}
#[wasm_bindgen_test]
fn raw_object_label_coerces_only_at_overflow_and_never_for_strict_equality() {
    let f = Fixture::new();
    let label =
        Function::new_no_args("return {calls:0,toString(){this.calls++;return 'localized';}}")
            .call0(&JsValue::UNDEFINED)
            .unwrap();
    f.labels.set_too_much_output_value(label.clone());
    assert_eq!(f.labels.too_much_output_value(), label);
    assert!(f.labels.too_much_output().is_err());
    assert_eq!(
        Reflect::get(&label, &"calls".into()).unwrap().as_f64(),
        Some(0.0)
    );
    for _ in 0..OVERFLOW_LINES {
        f.host.emit(Event::LineFeed).unwrap();
    }
    assert_eq!(text(&f.live()), units("localized"));
    assert_eq!(
        Reflect::get(&label, &"calls".into()).unwrap().as_f64(),
        Some(1.0)
    );
    f.host.emit(Event::Character(units("ignored"))).unwrap();
    f.render();
    let mut expected = units("localized");
    expected.extend([10; OVERFLOW_LINES]);
    assert_eq!(text(&f.live()), expected);
    assert_eq!(
        Reflect::get(&label, &"calls".into()).unwrap().as_f64(),
        Some(1.0)
    );
    f.host.emit(Event::Key(units("k"))).unwrap();
    f.host.emit(Event::Character(units("k"))).unwrap();
    f.host.emit(Event::Character(units("r"))).unwrap();
    f.render();
    assert_eq!(text(&f.live()), units("r"));
}
#[wasm_bindgen_test]
fn nonstring_primitive_labels_use_native_dom_coercion_without_false_string_equality() {
    let f = Fixture::new();
    for label in [
        JsValue::NULL,
        JsValue::UNDEFINED,
        JsValue::FALSE,
        JsValue::from_f64(123.0),
        JsValue::from_f64(f64::NAN),
        JsValue::from_f64(f64::INFINITY),
    ] {
        f.host.emit(Event::Blur).unwrap();
        f.labels.set_too_much_output_value(label.clone());
        let reference: web_sys::HtmlElement = f
            .document
            .create_element("div")
            .unwrap()
            .dyn_into()
            .unwrap();
        Reflect::set(&reference, &"textContent".into(), &label).unwrap();
        let mut expected = text(&reference);
        for _ in 0..OVERFLOW_LINES {
            f.host.emit(Event::LineFeed).unwrap();
        }
        assert_eq!(text(&f.live()), expected);
        expected.extend([10; OVERFLOW_LINES]);
        f.render();
        assert_eq!(text(&f.live()), expected);
        f.host.emit(Event::Character(units("dropped"))).unwrap();
        f.render();
        assert_eq!(text(&f.live()), expected);
    }
}
#[wasm_bindgen_test]
fn localized_lone_surrogates_survive_dom_replacement_and_flush_clear() {
    let f = Fixture::new();
    let label = js_sys::JsString::from_char_code(&[0xd800, 0xfeff]);
    f.labels.set_too_much_output_value(label.clone().into());
    for _ in 0..OVERFLOW_LINES {
        f.host.emit(Event::LineFeed).unwrap();
    }
    assert_eq!(text(&f.live()), [0xd800, 0xfeff]);
    f.render();
    assert_eq!(text(&f.live()), vec![10; OVERFLOW_LINES]);
    f.host.emit(Event::Character(vec![0xdc00, 0])).unwrap();
    f.render();
    let mut expected = vec![10; OVERFLOW_LINES];
    expected.extend([0xdc00, 0]);
    assert_eq!(text(&f.live()), expected);
}
