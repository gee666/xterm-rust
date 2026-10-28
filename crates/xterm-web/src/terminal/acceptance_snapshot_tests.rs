//! Read the actual engine snapshot, including values JSON cannot represent.
use super::*;
use js_sys::{Array, JsString, Object, Reflect};
use wasm_bindgen::JsCast;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn engine_marker_snapshot_preserves_raw_number_and_utf16_lines() {
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    let raw = [
        MarkerLine::number(-0.0),
        MarkerLine::number(0.0),
        MarkerLine::number(f64::NAN),
        MarkerLine::number(f64::INFINITY),
        MarkerLine::number(f64::NEG_INFINITY),
        MarkerLine::number(9_007_199_254_740_994.0),
        MarkerLine::from("0"),
        MarkerLine::string(vec![0xd800, 0x30, 0xdc00]),
    ];
    terminal
        .shared
        .native
        .with_state(|state| {
            for buffer in [state.buffers.normal(), state.buffers.alt()] {
                let mut buffer = buffer.borrow_mut();
                for line in &raw {
                    buffer.add_marker_value(line.clone());
                }
            }
        })
        .unwrap();
    let snapshot = terminal.acceptance_snapshot().unwrap();
    assert_eq!(Reflect::get(&snapshot, &"active".into()).unwrap(), "normal");
    for name in ["normal", "alternate"] {
        let buffer = Reflect::get(&snapshot, &name.into()).unwrap();
        let markers = Array::from(&Reflect::get(&buffer, &"markers".into()).unwrap());
        assert_eq!(markers.length() as usize, raw.len());
        for (marker, line) in markers.iter().zip(&raw) {
            let actual = Reflect::get(&marker, &"line".into()).unwrap();
            let expected = xterm_wasm_support::options::into_js(line.as_value()).unwrap();
            assert!(Object::is(&actual, &expected));
            if let Some(units) = line.as_string() {
                assert_eq!(
                    crate::utf16::from_js_string(&actual.dyn_into::<JsString>().unwrap()),
                    units
                );
            }
        }
    }
    terminal.dispose().unwrap();
}
