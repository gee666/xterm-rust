use super::*;
use js_sys::{Array, Object, Reflect};
use wasm_bindgen_test::*;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(inline_js = r#"
export function metricFixture() {
  const log = [];
  const family = { value: 'old', [Symbol.toPrimitive](hint) { log.push('family:' + hint); return this.value; } };
  const size = { [Symbol.toPrimitive](hint) { log.push('size:' + hint); family.value = 'new\ud800'; return '15e0'; } };
  return { family, size, log };
}
export function domFixture() {
  const log = [];
  const family = { [Symbol.toPrimitive](hint) { log.push('family:' + hint); return 'unexpected conversion'; } };
  const size = { [Symbol.toPrimitive](hint) { log.push('size:' + hint); return '15e0'; } };
  const element = document.createElement('span');
  let captured;
  const style = { set fontFamily(value) { captured = value; }, set fontSize(value) { log.push('fontSize:' + value); } };
  Object.defineProperty(element, 'style', { value: style });
  Object.defineProperty(element, 'offsetWidth', { get() { return '64'; } });
  Object.defineProperty(element, 'offsetHeight', { get() { return '10'; } });
  return { element, family, size, log, get captured() { return captured; } };
}
export function throwingProperty(target, error) {
  Object.defineProperty(target, 'font', { set(value) { target.captured = value; throw error; } });
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = metricFixture)]
    fn metric_fixture() -> JsValue;
    #[wasm_bindgen(js_name = domFixture)]
    fn dom_fixture() -> JsValue;
    #[wasm_bindgen(js_name = throwingProperty)]
    fn throwing_property(target: &JsValue, error: &JsValue);
}
fn property(value: &JsValue, name: &str) -> JsValue {
    Reflect::get(value, &JsValue::from_str(name)).unwrap()
}
fn options(fixture: &JsValue) -> Rc<OptionsService> {
    let options = Rc::new(OptionsService::new(false, [] as [(&str, OptionValue); 0]));
    for (key, name) in [("fontFamily", "family"), ("fontSize", "size")] {
        options
            .set_raw(
                key,
                xterm_wasm_support::options::from_js(&property(fixture, name)).unwrap(),
            )
            .unwrap();
    }
    options
}
fn log(fixture: &JsValue) -> Vec<String> {
    Array::from(&property(fixture, "log"))
        .iter()
        .map(|value| value.as_string().unwrap())
        .collect()
}

#[wasm_bindgen_test]
fn live_metrics_read_size_then_family_with_string_hint_and_raw_units() {
    let fixture = metric_fixture();
    let source = FontSource::Live(options(&fixture));
    let mut expected: Vec<_> = "15e0px new".encode_utf16().collect();
    expected.push(0xd800);
    assert_eq!(source.metrics_font().unwrap(), expected);
    assert_eq!(log(&fixture), ["size:string", "family:string"]);
}

#[wasm_bindgen_test]
fn dom_measurement_passes_family_object_to_setter_without_early_coercion() {
    let fixture = dom_fixture();
    // Fixture creates an actual span; only its own property behavior is customized.
    let element: HtmlElement = property(&fixture, "element").unchecked_into();
    let strategy = Strategy::Dom(element);
    let result = strategy
        .measure(&FontSource::Live(options(&fixture)))
        .unwrap();
    assert_eq!(result, (Some(2.0), Some(10.0)));
    assert_eq!(property(&fixture, "captured"), property(&fixture, "family"));
    assert_eq!(log(&fixture), ["size:string", "fontSize:15e0px"]);
}

#[wasm_bindgen_test]
fn strict_font_assignment_keeps_original_throw_and_lone_surrogate_value() {
    let target = Object::new();
    let error = Object::new();
    throwing_property(target.as_ref(), error.as_ref());
    let raw = [0xd800, 0, 0x7e, 97, 0xdc00];
    let value = crate::utf16::to_js_string(&raw);
    assert_eq!(
        crate::utf16::set_property(target.as_ref(), "font", value.as_ref()).unwrap_err(),
        JsValue::from(error)
    );
    let captured: js_sys::JsString = property(target.as_ref(), "captured").unchecked_into();
    assert_eq!(crate::utf16::from_js_string(&captured), raw);
}

#[wasm_bindgen_test]
fn iframe_canvas_fallback_retains_surrogate_comment_and_pixel_color() {
    let document = web_sys::window().unwrap().document().unwrap();
    let iframe: web_sys::HtmlIFrameElement =
        document.create_element("iframe").unwrap().unchecked_into();
    document.body().unwrap().append_child(&iframe).unwrap();
    let child = iframe.content_document().unwrap();
    let parser = crate::canvas_css::CanvasColorParser::new(&child).unwrap();
    let mut input: Vec<_> = "rgb(/*".encode_utf16().collect();
    input.push(0xd800);
    input.extend("*/255,0,0)".encode_utf16());
    let color = parser.to_color_utf16(&input).unwrap();
    assert_eq!(color.css_units(), input);
    assert_eq!(color.borrow().rgba, 0xff0000ff);
    iframe.remove();
}
