#![cfg(target_arch = "wasm32")]
// DomRenderer.ts creates WidthCache after CSS injection and refreshes fonts
// after each options-change CSS write. Exercise those callers, not just helpers.
use js_sys::{Array, JsString, Reflect};
use std::{cell::Cell, rc::Rc};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::{HtmlElement, Node};
use xterm_core::{
    headless::Terminal,
    options::{OptionValue, OptionsService},
};
use xterm_web::{
    char_size_browser::CharSizeService,
    character_joiner::CharacterJoinerService,
    dom::{BrowserDom, DomOptions},
    renderer::{dom::DomRenderer, width_cache::Font},
    theme::ThemeService,
    utf16::{from_js_string, to_js_string},
};
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/dom_font_probe.js")]
extern "C" {
    #[wasm_bindgen(js_name = installFontProbe)]
    fn install_probe(document: &web_sys::Document) -> JsValue;
    #[wasm_bindgen(js_name = fontValue)]
    fn font_value(probe: &JsValue, key: &str, text: &JsString) -> JsValue;
    #[wasm_bindgen(js_name = hookCss)]
    fn hook_css(element: &Node, callback: &js_sys::Function);
}
struct Fixture {
    renderer: Rc<DomRenderer>,
    options: Rc<OptionsService>,
    dom: BrowserDom,
    parent: HtmlElement,
    probe: JsValue,
}
fn field(object: &JsValue, key: &str) -> JsValue {
    Reflect::get(object, &key.into()).unwrap()
}
fn set(object: &JsValue, key: &str, value: &JsValue) {
    assert!(Reflect::set(object, &key.into(), value).unwrap());
}
fn raw(value: &JsValue) -> OptionValue {
    xterm_wasm_support::options::from_js(value).unwrap()
}
fn events(probe: &JsValue) -> Array {
    field(probe, "events").unchecked_into()
}
fn font_writes(probe: &JsValue) -> Vec<Vec<u16>> {
    events(probe)
        .iter()
        .filter_map(|event| {
            let event = Array::from(&event);
            (event.get(0).as_string().as_deref() == Some("font"))
                .then(|| from_js_string(&JsString::from(event.get(2))))
        })
        .collect()
}
fn clear_events(probe: &JsValue) {
    events(probe).set_length(0);
}
fn source_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!("fixtures/dom-font.json")).unwrap()
}
fn font_trace(probe: &JsValue) -> serde_json::Value {
    events(probe)
        .iter()
        .map(|event| {
            let event = Array::from(&event);
            let kind = event.get(0).as_string().unwrap();
            match kind.as_str() {
                "coerce" => serde_json::json!([
                    kind,
                    event.get(1).as_string().unwrap(),
                    event.get(2).as_string().unwrap()
                ]),
                "font" => serde_json::json!([
                    kind,
                    event.get(1).as_f64().unwrap() as usize,
                    from_js_string(&JsString::from(event.get(2)))
                ]),
                _ => serde_json::json!([kind, event.get(1).as_f64().unwrap() as usize]),
            }
        })
        .collect()
}
fn family() -> Vec<u16> {
    let mut units: Vec<_> = "Face".encode_utf16().collect();
    units.extend([0xd800, 0, 0xdc00]);
    units
}
fn fixture() -> Fixture {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent: HtmlElement = document.create_element("div").unwrap().unchecked_into();
    document.body().unwrap().append_child(&parent).unwrap();
    let dom = BrowserDom::new(&parent, &DomOptions::default()).unwrap();
    let metrics = Rc::new(CharSizeService::new(&document, &dom.helpers, Font::default()).unwrap());
    metrics
        .state
        .update_measurement(Some(8.0), Some(16.0))
        .unwrap();
    let terminal = Rc::new(Terminal::new(4, 2, 0).unwrap());
    let options = terminal
        .with_state(|state| Rc::clone(&state.options))
        .unwrap();
    let probe = install_probe(&document);
    for (key, text) in [
        ("fontFamily", to_js_string(&family())),
        ("fontSize", JsString::from("15e0")),
        ("fontWeight", JsString::from("normal")),
        ("fontWeightBold", JsString::from("bold")),
    ] {
        options
            .set_raw(key, raw(&font_value(&probe, key, &text)))
            .unwrap();
    }
    let decorations = terminal
        .with_state(|state| {
            xterm_core::decoration::DecorationService::with_buffers(&state.buffers).unwrap()
        })
        .unwrap();
    let renderer = DomRenderer::new_with_metrics(
        terminal,
        &dom,
        Rc::new(ThemeService::default()),
        Rc::new(CharacterJoinerService::new(|error| {
            panic!("unexpected joiner error: {error}")
        })),
        decorations,
        metrics,
    )
    .unwrap();
    Fixture {
        renderer,
        options,
        dom,
        parent,
        probe,
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.renderer.dispose();
        field(&self.probe, "restore")
            .unchecked_into::<js_sys::Function>()
            .call0(&self.probe)
            .unwrap();
        self.parent.remove();
    }
}
fn css(fixture: &Fixture) -> Node {
    fixture
        .dom
        .screen
        .query_selector_all("style")
        .unwrap()
        .item(1)
        .unwrap()
}
#[wasm_bindgen_test]
fn actual_constructor_preserves_raw_fonts_and_has_no_eager_font_projection() {
    let f = fixture();
    assert_eq!(font_trace(&f.probe), source_fixture()["constructor"]);
    let mut expected = Vec::new();
    for key in ["fontFamily", "fontSize", "fontWeight", "fontWeightBold"] {
        expected.push(format!("coerce:{key}:string"));
    }
    expected.extend((0..4).map(|index| format!("canvas:{index}")));
    for (index, weight) in [
        "fontWeight",
        "fontWeightBold",
        "fontWeight",
        "fontWeightBold",
    ]
    .into_iter()
    .enumerate()
    {
        for key in [weight, "fontSize", "fontFamily"] {
            expected.push(format!("coerce:{key}:string"));
        }
        expected.push(format!("font:{index}"));
    }
    let actual: Vec<_> = events(&f.probe)
        .iter()
        .map(|event| {
            let event = Array::from(&event);
            match event.get(0).as_string().unwrap().as_str() {
                "coerce" => format!(
                    "coerce:{}:{}",
                    event.get(1).as_string().unwrap(),
                    event.get(2).as_string().unwrap()
                ),
                kind => format!("{kind}:{}", event.get(1).as_f64().unwrap()),
            }
        })
        .collect();
    assert_eq!(actual, expected);
    let writes = font_writes(&f.probe);
    for (index, prefix) in ["normal", "bold", "italic normal", "italic bold"]
        .into_iter()
        .enumerate()
    {
        let mut expected: Vec<_> = format!("{prefix} 15e0px ").encode_utf16().collect();
        expected.extend(family());
        assert_eq!(writes[index], expected);
    }
    let text: JsString = field(css(&f).as_ref(), "textContent").unchecked_into();
    assert!(from_js_string(&text)
        .windows(family().len())
        .any(|part| part == family()));
}
#[wasm_bindgen_test]
fn css_write_can_change_the_font_used_by_the_following_canvas_setters() {
    let f = fixture();
    clear_events(&f.probe);
    let options = Rc::clone(&f.options);
    let callback = Closure::wrap(Box::new(move |_: JsValue| {
        options
            .set_raw("fontFamily", OptionValue::String(vec![0xd800, 97, 0xdc00]))
            .unwrap();
    }) as Box<dyn FnMut(JsValue)>);
    hook_css(&css(&f), callback.as_ref().unchecked_ref());
    f.options.set("cursorBlink", true.into()).unwrap();
    let writes = font_writes(&f.probe);
    let actual: Vec<_> = font_trace(&f.probe)
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry[0] == "font")
        .cloned()
        .collect();
    assert_eq!(
        serde_json::Value::Array(actual),
        source_fixture()["afterCssMutation"]
    );
    assert_eq!(writes.len(), 4);
    for value in writes {
        assert!(value.ends_with(&[0xd800, 97, 0xdc00]));
    }
    // No JS callback remains installed after its Rust closure is dropped.
    assert!(Reflect::delete_property(css(&f).unchecked_ref(), &"textContent".into()).unwrap());
}
#[wasm_bindgen_test]
fn canvas_font_setters_can_reenter_renderer_measurement_without_borrow_panics() {
    let f = fixture();
    let weak = Rc::downgrade(&f.renderer);
    let calls = Rc::new(Cell::new(0_u32));
    let count = Rc::clone(&calls);
    let callback = Closure::wrap(Box::new(move || {
        count.set(count.get() + 1);
        weak.upgrade().unwrap().handle_char_size_changed().unwrap();
    }) as Box<dyn FnMut()>);
    set(&f.probe, "onFont", callback.as_ref());
    f.options.set_raw("fontFamily", "changed".into()).unwrap();
    f.options.set("cursorBlink", true.into()).unwrap();
    assert_eq!(
        u64::from(calls.get()),
        source_fixture()["reentrantCalls"].as_u64().unwrap()
    );
    assert_eq!(calls.get(), 4);
    f.renderer.render_rows(0, 1).unwrap();
    set(&f.probe, "onFont", &JsValue::UNDEFINED);
}
#[wasm_bindgen_test]
fn failing_live_css_write_returns_the_original_error_before_touching_canvases() {
    let f = fixture();
    clear_events(&f.probe);
    let error = js_sys::Object::new();
    let callback =
        js_sys::Function::new_with_args("error", "return function(value) { throw error; }")
            .call1(&JsValue::UNDEFINED, &error)
            .unwrap()
            .unchecked_into::<js_sys::Function>();
    hook_css(&css(&f), &callback);
    let actual = f.options.set("cursorBlink", true.into()).unwrap_err();
    assert_eq!(
        xterm_wasm_support::js_error::to_js_error(&actual),
        JsValue::from(error)
    );
    assert_eq!(
        font_writes(&f.probe).len() as u64,
        source_fixture()["failedCssCanvasWrites"].as_u64().unwrap()
    );
    assert!(font_writes(&f.probe).is_empty());
    assert_eq!(f.options.get("cursorBlink").unwrap(), true.into());
    assert!(Reflect::delete_property(css(&f).unchecked_ref(), &"textContent".into()).unwrap());
}
