#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::utf16::{from_js_string, to_js_string};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/dom_font_probe.js")]
extern "C" {
    #[wasm_bindgen(js_name = installFontProbe)]
    fn install_probe(document: &web_sys::Document) -> JsValue;
}
fn field(object: &JsValue, name: &str) -> JsValue {
    Reflect::get(object, &name.into()).unwrap()
}
#[wasm_bindgen_test]
fn opened_public_terminal_keeps_raw_font_css_and_canvas_units_after_live_updates() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent: HtmlElement = document.create_element("div").unwrap().unchecked_into();
    document.body().unwrap().append_child(&parent).unwrap();
    let probe = install_probe(&document);
    let options = Object::new();
    for (key, value) in [("cols", 4.0), ("rows", 2.0)] {
        assert!(Reflect::set(&options, &key.into(), &value.into()).unwrap());
    }
    let terminal = xterm_web::Terminal::new(options.into()).unwrap();
    terminal.open(parent.clone().into()).unwrap();
    let events: Array = field(&probe, "events").unchecked_into();
    events.set_length(0);
    let mut units: Vec<_> = "monospace/*".encode_utf16().collect();
    units.extend([0xd800, 0, 0xdc00]);
    units.extend("*/".encode_utf16());
    assert!(Reflect::set(
        &terminal.options(),
        &"fontFamily".into(),
        &to_js_string(&units)
    )
    .unwrap());
    let styles = terminal
        .element()
        .unwrap()
        .query_selector_all("style")
        .unwrap();
    let theme_style = styles.item(1).unwrap();
    let text: js_sys::JsString = field(theme_style.as_ref(), "textContent").unchecked_into();
    assert!(from_js_string(&text)
        .windows(units.len())
        .any(|part| part == units));
    let writes: Vec<_> = events
        .iter()
        .filter_map(|event| {
            let event = Array::from(&event);
            (event.get(0).as_string().as_deref() == Some("font"))
                .then(|| from_js_string(&js_sys::JsString::from(event.get(2))))
        })
        .collect();
    assert_eq!(writes.len(), 4);
    for value in writes {
        assert!(value.ends_with(&units));
    }
    assert_eq!(
        field(terminal.options().as_ref(), "fontFamily"),
        JsValue::from(to_js_string(&units))
    );
    terminal.dispose().unwrap();
    assert_eq!(parent.child_element_count(), 0);
    field(&probe, "restore")
        .unchecked_into::<js_sys::Function>()
        .call0(&probe)
        .unwrap();
    parent.remove();
}
