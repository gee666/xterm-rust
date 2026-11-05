#![cfg(target_arch = "wasm32")]
//! Raw numeric keypress fields through the real Rust Terminal DOM listener.
use js_sys::{Array, Function, Object, Reflect};
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/fixtures/keypress-cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runKeypressCase)]
    fn run_case(
        terminal: &JsValue,
        bodies: &Array,
        kind: &str,
        route: &str,
        modifier: &str,
        behavior: &str,
    ) -> Result<JsValue, JsValue>;
}
struct Setup {
    terminal: JsValue,
    parent: web_sys::Element,
    style: web_sys::Element,
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
        Reflect::set(&options, &"cols".into(), &10.into()).unwrap();
        Reflect::set(&options, &"rows".into(), &3.into()).unwrap();
        Reflect::set(&options, &"logLevel".into(), &"off".into()).unwrap();
        let terminal = Terminal::new(options.into()).unwrap();
        terminal.open(parent.clone().into()).unwrap();
        Self {
            terminal: terminal.into(),
            parent,
            style,
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
        include_str!("fixtures/keypress-body.js"),
        include_str!("fixtures/third-level-shift-body.js"),
        include_str!("fixtures/input-event-body.js"),
    ]
    .into_iter()
    .map(JsValue::from_str)
    .collect()
}

#[wasm_bindgen_test]
fn raw_charcode_which_and_fallback_keycode_match_unchanged_source() {
    let bodies = bodies();
    for kind in [
        "undefined",
        "null",
        "false",
        "true",
        "zero",
        "negative-zero",
        "nan",
        "empty",
        "string-zero",
        "string-number",
        "string-surrogate-number",
        "whitespace",
        "surrogate",
        "symbol",
        "bigint",
        "array",
        "function",
        "boxed",
        "infinity",
        "object",
    ] {
        for route in ["char", "changed-char", "which", "fallback"] {
            for modifier in ["none", "alt", "ctrl", "meta"] {
                let setup = Setup::new();
                run_case(&setup.terminal, &bodies, kind, route, modifier, "plain").unwrap();
            }
        }
    }
}

#[wasm_bindgen_test]
fn conversion_getter_and_callback_errors_and_reentry_preserve_source_flags() {
    let bodies = bodies();
    for behavior in [
        "conversion-error",
        "throw-undefined",
        "conversion-nested",
        "conversion-change",
        "payload-getter-error",
        "key-error",
        "key-nested",
        "data-error",
    ] {
        for route in ["char", "which", "fallback"] {
            let setup = Setup::new();
            run_case(&setup.terminal, &bodies, "object", route, "none", behavior).unwrap();
        }
    }
}
