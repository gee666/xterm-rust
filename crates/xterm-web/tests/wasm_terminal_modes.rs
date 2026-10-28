#![cfg(target_arch = "wasm32")]

use js_sys::{Object, Reflect};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

fn property(value: &JsValue, name: &str) -> JsValue {
    Reflect::get(value, &JsValue::from_str(name)).unwrap()
}

#[wasm_bindgen_test]
fn public_modes_are_complete_fresh_owned_snapshots() {
    let options = js_sys::JSON::parse(r#"{"vtExtensions":{"win32InputMode":true}}"#).unwrap();
    let terminal = Terminal::new(options).unwrap();
    let first = terminal.modes().unwrap();
    assert_eq!(Object::keys(&Object::from(first.clone())).length(), 12);
    for name in [
        "applicationCursorKeysMode",
        "applicationKeypadMode",
        "bracketedPasteMode",
        "insertMode",
        "originMode",
        "reverseWraparoundMode",
        "sendFocusMode",
        "synchronizedOutputMode",
        "win32InputMode",
    ] {
        assert_eq!(property(&first, name), JsValue::FALSE, "{name}");
    }
    assert_eq!(property(&first, "showCursor"), JsValue::TRUE);
    assert_eq!(property(&first, "wraparoundMode"), JsValue::TRUE);
    assert_eq!(
        property(&first, "mouseTrackingMode"),
        JsValue::from_str("none")
    );
    terminal
        .shared_native()
        .write("\x1b[?1;66;2004;6;45;1004;2026;9001h\x1b[4h\x1b[?25;7l")
        .unwrap();
    let changed = terminal.modes().unwrap();
    for name in [
        "applicationCursorKeysMode",
        "applicationKeypadMode",
        "bracketedPasteMode",
        "insertMode",
        "originMode",
        "reverseWraparoundMode",
        "sendFocusMode",
        "synchronizedOutputMode",
        "win32InputMode",
    ] {
        assert_eq!(property(&changed, name), JsValue::TRUE, "{name}");
        assert_eq!(property(&first, name), JsValue::FALSE, "old {name}");
    }
    assert_eq!(property(&changed, "showCursor"), JsValue::FALSE);
    assert_eq!(property(&changed, "wraparoundMode"), JsValue::FALSE);
    Reflect::set(&changed, &JsValue::from_str("insertMode"), &JsValue::FALSE).unwrap();
    assert_eq!(
        property(&terminal.modes().unwrap(), "insertMode"),
        JsValue::TRUE
    );
    terminal.dispose().unwrap();
}

#[wasm_bindgen_test]
fn public_mouse_modes_follow_the_live_protocol() {
    let terminal = Terminal::new(JsValue::UNDEFINED).unwrap();
    for (mode, expected) in [(9, "x10"), (1000, "vt200"), (1002, "drag"), (1003, "any")] {
        terminal
            .shared_native()
            .write(&format!("\x1b[?{mode}h"))
            .unwrap();
        assert_eq!(
            property(&terminal.modes().unwrap(), "mouseTrackingMode"),
            JsValue::from_str(expected)
        );
    }
    terminal.shared_native().write("\x1b[?9l").unwrap();
    assert_eq!(
        property(&terminal.modes().unwrap(), "mouseTrackingMode"),
        JsValue::from_str("none")
    );
    terminal.dispose().unwrap();
}
