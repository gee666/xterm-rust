#![cfg(target_arch = "wasm32")]
use js_sys::{Array, Function, JsString, Object, Reflect};
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_core::buffer::{AttributeData, BufferLine};
use xterm_web::character_joiner::{
    raw::{report_error, BrowserJoinerService, RawRanges},
    LineRef,
};
wasm_bindgen_test_configure!(run_in_browser);
#[path = "support/cells.rs"]
mod cells;

#[wasm_bindgen(module = "/tests/support/joiner_raw.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = joinerRawScenarios)]
    fn scenarios(create: &Function) -> Result<JsString, JsValue>;
    #[wasm_bindgen(js_name = liveJoinerFixture)]
    fn live_fixture() -> JsValue;
}

#[wasm_bindgen]
pub struct RawJoinerProbe {
    service: BrowserJoinerService,
    line: LineRef,
}
#[wasm_bindgen]
impl RawJoinerProbe {
    pub fn register(&self, handler: Function) -> f64 {
        self.service.register_js(handler)
    }
    pub fn deregister(&self, id: f64) -> Result<bool, JsValue> {
        self.service.deregister_js(id)
    }
    pub fn get(&self) -> Result<JsValue, JsValue> {
        self.service
            .try_get_joined_characters(Some(Rc::clone(&self.line)))
            .map(|ranges| ranges.0)
    }
}
fn probe() -> RawJoinerProbe {
    let mut line = BufferLine::new(6, None, false);
    for (index, cp, width) in [
        (0, 97, 1),
        (1, 0x4e2d, 2),
        (2, 0, 0),
        (3, 98, 1),
        (4, 99, 1),
        (5, 100, 1),
    ] {
        line.set_cell_from_codepoint(index, cp, width, &AttributeData::default());
    }
    RawJoinerProbe {
        service: BrowserJoinerService::with_error_reporter(report_error),
        line: Rc::new(RefCell::new(line)),
    }
}
#[wasm_bindgen_test]
fn raw_ranges_match_unchanged_source_identity_errors_and_operator_order() {
    let create =
        Closure::wrap(Box::new(|| -> JsValue { probe().into() }) as Box<dyn Fn() -> JsValue>);
    let actual = scenarios(create.as_ref().unchecked_ref()).unwrap();
    let actual: serde_json::Value = serde_json::from_str(&String::from(actual)).unwrap();
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/joiner-raw.json")).unwrap();
    assert_eq!(actual, expected);
}
#[wasm_bindgen_test]
fn raw_service_replays_all_original_captured_calls() {
    use std::collections::VecDeque;
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/character-joiner.json")).unwrap();
    let cases = fixture["calls"].as_array().unwrap();
    assert_eq!(cases.len(), 27);
    for (index, case) in cases.iter().enumerate() {
        let service = BrowserJoinerService::new(|error| panic!("{error:?}"));
        let mut queues = Vec::new();
        for id in case["handlers"].as_array().unwrap() {
            let queue: VecDeque<_> = case["handlerCalls"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|call| call["id"] == *id)
                .cloned()
                .collect();
            let queue = Rc::new(RefCell::new(queue));
            queues.push(Rc::clone(&queue));
            service.register(Rc::new(move |text| {
                let call = queue
                    .borrow_mut()
                    .pop_front()
                    .expect("source callback exists");
                let expected: Vec<_> = call["text"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|unit| cells::number(unit) as u16)
                    .collect();
                assert_eq!(text, expected);
                Ok(RawRanges(
                    js_sys::JSON::parse(&call["ranges"].to_string()).unwrap(),
                ))
            }));
        }
        let line = case["cells"]
            .as_array()
            .map(|raw| Rc::new(RefCell::new(cells::line(raw))));
        let raw = service.try_get_joined_characters(line).unwrap();
        let actual: serde_json::Value = serde_json::from_str(
            &js_sys::JSON::stringify(&raw.0)
                .unwrap()
                .as_string()
                .unwrap(),
        )
        .unwrap();
        assert_eq!(actual, case["expected"], "original call {index}");
        assert!(queues.iter().all(|queue| queue.borrow().is_empty()));
    }
}
fn field(value: &JsValue, key: &str) -> JsValue {
    Reflect::get(value, &key.into()).unwrap()
}
fn method(value: &JsValue, key: &str) -> Function {
    field(value, key).dyn_into().unwrap()
}
#[wasm_bindgen_test]
fn actual_terminal_keeps_raw_inner_ranges_and_conversion_error_identity() {
    let document = web_sys::window().unwrap().document().unwrap();
    let parent = document.create_element("div").unwrap();
    document.body().unwrap().append_child(&parent).unwrap();
    let options = Object::new();
    Reflect::set(&options, &"cols".into(), &10.into()).unwrap();
    Reflect::set(&options, &"rows".into(), &2.into()).unwrap();
    let terminal = xterm_web::Terminal::new(options.into()).unwrap();
    terminal.open(parent.clone().into()).unwrap();
    terminal.shared_native().write("a中bcd").unwrap();
    let fixture = live_fixture();
    let renderer = terminal
        .render_service_handle()
        .unwrap()
        .current_renderer()
        .unwrap();
    let id = terminal
        .register_character_joiner(method(&fixture, "handler"))
        .unwrap();
    renderer.render_rows(0.0, 0.0).unwrap();
    let first = Array::from(&field(&fixture, "first"));
    let inner = Array::from(&first.get(0));
    assert_eq!(inner.get(0).as_f64(), Some(1.0));
    assert_eq!(inner.get(1).as_f64(), Some(4.0));
    let spans = parent
        .query_selector_all(".xterm-rows > :first-child > span")
        .unwrap();
    let texts: Vec<_> = (0..spans.length())
        .map(|i| spans.item(i).unwrap().text_content().unwrap())
        .collect();
    assert!(texts.contains(&"中b".to_owned()), "{texts:?}");
    terminal.deregister_character_joiner(id).unwrap();
    let bad = terminal
        .register_character_joiner(method(&fixture, "failingConversion"))
        .unwrap();
    let error = renderer.render_rows(0.0, 0.0).unwrap_err();
    assert_eq!(error, field(&fixture, "marker"));
    terminal.deregister_character_joiner(bad).unwrap();
    method(&fixture, "installLogger").call0(&fixture).unwrap();
    let throwing = terminal
        .register_character_joiner(method(&fixture, "failingHandler"))
        .unwrap();
    let outcome = renderer.render_rows(0.0, 0.0);
    method(&fixture, "restoreLogger").call0(&fixture).unwrap();
    outcome.unwrap();
    assert_eq!(
        Array::from(&field(&fixture, "logged")).get(0),
        field(&fixture, "marker")
    );
    terminal.deregister_character_joiner(throwing).unwrap();
    terminal.dispose().unwrap();
    parent.remove();
}
