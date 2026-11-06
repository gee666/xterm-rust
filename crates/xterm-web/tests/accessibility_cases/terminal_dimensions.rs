// MIT. Dimensions errors must cross the actual opened Terminal service wiring.
use super::Opened;
use js_sys::{Function, Reflect};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::BrowserRuntime;

#[wasm_bindgen(module = "/tests/accessibility_dimensions_oracle.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = rustDimensionsCase)]
    fn rust_case(
        container: &HtmlElement,
        mode: &str,
        refresh: &Function,
        change: &Function,
    ) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen_test]
fn actual_terminal_resize_propagates_accessibility_css_assignment_error() {
    let fixture = Opened::new(&BrowserRuntime::new()).unwrap();
    let container = fixture
        .terminal
        .element()
        .unwrap()
        .query_selector(".xterm-accessibility")
        .unwrap()
        .unwrap()
        .unchecked_into::<HtmlElement>();
    let terminal = Rc::clone(&fixture.terminal);
    let resize =
        Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || terminal.resize(10.0, 3.0));
    let unexpected_change =
        Function::new_no_args("throw new Error('Unexpected dimension mutation')");
    let result = rust_case(
        &container,
        "width-throw",
        resize.as_ref().unchecked_ref(),
        &unexpected_change,
    )
    .unwrap();
    assert_eq!(
        Reflect::get(&result, &"failure".into())
            .unwrap()
            .as_string()
            .as_deref(),
        Some("marker")
    );
    assert_eq!(fixture.terminal.cols().unwrap(), 10.0);
}
