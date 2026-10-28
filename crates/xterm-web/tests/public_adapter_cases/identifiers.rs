//! Getter/proxy traces and real dispatch against unchanged source observations.
use js_sys::{Array, Function, JsString, Reflect};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_core::headless::Terminal;
use xterm_web::{terminal::parser_api::WebParserApi, utf16::from_js_string};

#[wasm_bindgen(module = "/tests/fixtures/parser_identifiers.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = identifierSpecs)]
    fn identifier_specs() -> Array;
    #[wasm_bindgen(catch, js_name = makeIdentifier)]
    fn make_identifier(spec: &JsValue, log: &Array, hook: &Function) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = registerIdentifier)]
    fn register_identifier(
        api: &JsValue,
        spec: &JsValue,
        id: &JsValue,
        calls: &Array,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(js_name = identifierHook)]
    fn identifier_hook(api: &JsValue, calls: &Array) -> Function;
    #[wasm_bindgen(js_name = identifierObservation)]
    fn observation(
        name: &str,
        log: &Array,
        calls: &Array,
        failed: bool,
        error: &JsValue,
        sentinel: &JsValue,
    ) -> JsValue;
}
fn field(object: &JsValue, name: &str) -> JsValue {
    Reflect::get(object, &name.into()).unwrap()
}
fn json(value: &JsValue) -> String {
    js_sys::JSON::stringify(value).unwrap().as_string().unwrap()
}

#[wasm_bindgen_test]
fn source_identifier_access_logs_errors_and_dispatch_match_without_snapshots() {
    let specs = identifier_specs();
    let expected = Array::from(
        &js_sys::JSON::parse(include_str!("../fixtures/parser-identifiers.json")).unwrap(),
    );
    assert_eq!(specs.length(), expected.length());
    for index in 0..specs.length() {
        let spec = specs.get(index);
        let name = field(&spec, "name").as_string().unwrap();
        let terminal = Rc::new(Terminal::new(20, 3, 10).unwrap());
        let api: JsValue = WebParserApi::new(terminal.clone()).into();
        let log = Array::new();
        let calls = Array::new();
        let fixture = make_identifier(&spec, &log, &identifier_hook(&api, &calls)).unwrap();
        let (failed, error) = match register_identifier(&api, &spec, &field(&fixture, "id"), &calls)
        {
            Ok(_) => (false, JsValue::UNDEFINED),
            Err(error) => (true, error),
        };
        let sequence = field(&spec, "sequence").dyn_into::<JsString>().unwrap();
        terminal.write_utf16(&from_js_string(&sequence)).unwrap();
        let actual = observation(
            &name,
            &log,
            &calls,
            failed,
            &error,
            &field(&fixture, "sentinel"),
        );
        assert_eq!(
            json(&actual),
            json(&expected.get(index)),
            "source case {name}"
        );
        terminal.dispose().unwrap();
    }
}
