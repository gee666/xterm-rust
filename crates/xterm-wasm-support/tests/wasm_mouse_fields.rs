#![cfg(target_arch = "wasm32")]
use js_sys::{Object, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_core::mouse_fields::{Encoding, Protocol};
use xterm_wasm_support::mouse_fields::{encoder, restriction};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/mouse_fields_cases.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = compareAll)]
    fn compare_all(protocols: &Object, encodings: &Object, fixtures: &str) -> Result<u32, JsValue>;
}

#[wasm_bindgen_test]
fn every_raw_builtin_read_coercion_and_failure_matches_source() {
    let protocols = Object::new();
    for (name, policy) in [
        ("NONE", Protocol::None),
        ("X10", Protocol::X10),
        ("VT200", Protocol::Vt200),
        ("DRAG", Protocol::Drag),
        ("ANY", Protocol::Any),
    ] {
        Reflect::set(&protocols, &name.into(), &restriction(policy)).unwrap();
    }
    let encodings = Object::new();
    for (name, policy) in [
        ("DEFAULT", Encoding::Default),
        ("SGR", Encoding::Sgr),
        ("SGR_PIXELS", Encoding::SgrPixels),
    ] {
        Reflect::set(&encodings, &name.into(), &encoder(policy)).unwrap();
    }
    assert_eq!(
        compare_all(
            &protocols,
            &encodings,
            include_str!("reference/mouse-fields.json")
        )
        .unwrap(),
        1125
    );
}
