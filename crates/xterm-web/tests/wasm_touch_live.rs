#![cfg(target_arch = "wasm32")]
use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_test::*;
use web_sys::{Document, HtmlElement};
use xterm_core::lifecycle::DisposableHandle;
use xterm_web::input::touch::GestureContext;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn original_touch_getter_event_and_partial_history_traces_match_rust_dom() {
    let corpus: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/touch-live.json")).unwrap();
    let cases = corpus.as_array().unwrap();
    assert_eq!(cases.len(), 91);
    let create = js_sys::Function::new_no_args(concat!(
        include_str!("../../../tools/touch_live/browser_host.js"),
        "\nreturn createBrowserTouchHost();"
    ));
    let run = js_sys::Function::new_with_args("host,name,cleanup", concat!(include_str!("../../../tools/touch_live/driver.js"),
        "\ntry { return JSON.stringify(runTouchScenario(host,name)); } finally { try { cleanup(); } finally { host.restore(); } }"));
    for case in cases {
        let name = case["name"].as_str().unwrap();
        let host = create.call0(&JsValue::UNDEFINED).unwrap();
        let document: Document = js_sys::Reflect::get(&host, &"document".into())
            .unwrap()
            .dyn_into()
            .unwrap();
        let context = GestureContext::new(document).unwrap();
        let mut handles = Vec::<DisposableHandle>::new();
        for label in case["targets"].as_array().unwrap() {
            let element: HtmlElement = js_sys::Reflect::get(&host, &label.as_str().unwrap().into())
                .unwrap()
                .dyn_into()
                .unwrap();
            handles.push(context.add_target(&element).unwrap());
        }
        for label in case["ignores"].as_array().unwrap() {
            let element: HtmlElement = js_sys::Reflect::get(&host, &label.as_str().unwrap().into())
                .unwrap()
                .dyn_into()
                .unwrap();
            handles.push(context.ignore_target(&element).unwrap());
        }
        let context = Rc::new(RefCell::new(Some(context)));
        let captured = Rc::clone(&context);
        let handles = Rc::new(RefCell::new(handles));
        let captured_handles = Rc::clone(&handles);
        let cleanup = Closure::wrap(Box::new(move || -> Result<(), JsValue> {
            let handles = std::mem::take(&mut *captured_handles.borrow_mut());
            for handle in handles {
                handle.dispose();
            }
            let context = captured.borrow_mut().take();
            if let Some(context) = context {
                context.dispose()?;
            }
            Ok(())
        }) as Box<dyn Fn() -> Result<(), JsValue>>);
        let result = run
            .call3(&JsValue::UNDEFINED, &host, &name.into(), cleanup.as_ref())
            .unwrap();
        let actual: Vec<String> = serde_json::from_str(&result.as_string().unwrap()).unwrap();
        let expected: Vec<String> = case["log"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry.as_str().unwrap().to_owned())
            .collect();
        if actual != expected {
            let index = (0..actual.len().max(expected.len()))
                .find(|&index| actual.get(index) != expected.get(index))
                .unwrap();
            panic!("touch live case {name}, first difference {index}: actual {:?}, source {:?}; lengths {}/{}", actual.get(index), expected.get(index), actual.len(), expected.len());
        }
        assert!(
            context.borrow().is_none(),
            "JS finally must drop the context before restoring clock/RAF"
        );
    }
}
