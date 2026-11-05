#![cfg(target_arch = "wasm32")]
//! Complete Rust Terminal routing compared with unchanged InputEvent, CoreService
//! and LogService bodies. Native subscriptions observe scroll/user-input order.
use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_core::lifecycle::DisposableHandle;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/fixtures/raw-input-cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runRawInputCase)]
    fn run_case(
        terminal: &JsValue,
        trace: &Array,
        bodies: &Array,
        kind: &str,
        behavior: &str,
        level: &str,
        disabled: bool,
    ) -> Result<JsValue, JsValue>;
}

struct Setup {
    terminal: JsValue,
    parent: web_sys::Element,
    style: web_sys::Element,
    trace: Array,
    _listeners: Vec<DisposableHandle>,
}
impl Setup {
    fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let style = document.create_element("style").unwrap();
        style.set_text_content(Some(xterm_web::XTERM_CSS));
        document.body().unwrap().append_child(&style).unwrap();
        let parent = document.create_element("div").unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let options = Object::new();
        for (key, value) in [("cols", 10), ("rows", 3), ("scrollback", 8)] {
            Reflect::set(&options, &key.into(), &value.into()).unwrap();
        }
        Reflect::set(&options, &"logLevel".into(), &"off".into()).unwrap();
        let terminal = Terminal::new(options.into()).unwrap();
        terminal.open(parent.clone().into()).unwrap();
        let native = terminal.shared_native();
        native.write("a\r\nb\r\nc\r\nd\r\ne").unwrap();
        native.scroll_to_top().unwrap();
        let core = native.with_state(|state| state.core.clone()).unwrap();
        let trace = Array::new();
        let output = trace.clone();
        let scroll = core.on_request_scroll_to_bottom().subscribe(move |_| {
            output.push(&"scroll".into());
        });
        let output = trace.clone();
        let user = core.on_user_input().subscribe(move |_| {
            output.push(&"user".into());
        });
        Self {
            terminal: terminal.into(),
            parent,
            style,
            trace,
            _listeners: vec![scroll, user],
        }
    }
}
impl Drop for Setup {
    fn drop(&mut self) {
        let dispose: Function = Reflect::get(&self.terminal, &"dispose".into())
            .unwrap()
            .unchecked_into();
        dispose.call0(&self.terminal).unwrap();
        self.parent.remove();
        self.style.remove();
    }
}
fn bodies() -> Array {
    [
        include_str!("fixtures/input-event-body.js"),
        include_str!("fixtures/raw-input-triggerDataEvent.js"),
        include_str!("fixtures/raw-input-evalLazy.js"),
        include_str!("fixtures/raw-input-log.js"),
        include_str!("fixtures/raw-input-trace.js"),
        include_str!("fixtures/raw-input-debug.js"),
    ]
    .into_iter()
    .map(JsValue::from_str)
    .collect()
}

#[wasm_bindgen_test]
fn raw_second_payloads_keep_identity_and_source_log_admission_order() {
    let bodies = bodies();
    for kind in [
        "null",
        "undefined",
        "false",
        "zero",
        "nan",
        "negativeZero",
        "bigint",
        "symbol",
        "array",
        "function",
        "object",
    ] {
        for level in ["off", "debug", "trace"] {
            for disabled in [false, true] {
                let setup = Setup::new();
                run_case(
                    &setup.terminal,
                    &setup.trace,
                    &bodies,
                    kind,
                    "plain",
                    level,
                    disabled,
                )
                .unwrap();
            }
        }
    }
}

#[wasm_bindgen_test]
fn raw_object_coercion_log_callbacks_and_reentry_match_source() {
    let bodies = bodies();
    for behavior in [
        "data-throw",
        "data-nested",
        "coercion-throw",
        "coercion-nested",
        "split-throw",
        "logger-getter-throw",
        "debug-enables-trace",
        "dispose",
    ] {
        for level in ["off", "debug", "trace"] {
            let setup = Setup::new();
            run_case(
                &setup.terminal,
                &setup.trace,
                &bodies,
                "object",
                behavior,
                level,
                false,
            )
            .unwrap();
        }
    }
}
