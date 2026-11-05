#![cfg(target_arch = "wasm32")]
use js_sys::{Function, Reflect};
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use xterm_core::buffer::AttributeData;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen(module = "/tests/support/joiner_row_live.mjs")]
extern "C" {
    #[wasm_bindgen(js_name = liveRowFixture)]
    fn live_fixture(mode: &str, on_read: &JsValue) -> JsValue;
}
fn field(value: &JsValue, key: &str) -> JsValue {
    Reflect::get(value, &key.into()).unwrap()
}
#[wasm_bindgen_test]
fn live_terminal_reads_only_reached_ranges_and_preserves_late_error_identity() {
    use wasm_bindgen::JsCast;
    let document = web_sys::window().unwrap().document().unwrap();
    for mode in ["unmatched", "fractional", "mutation", "error", "disposal"] {
        let parent = document.create_element("div").unwrap();
        document.body().unwrap().append_child(&parent).unwrap();
        let options = js_sys::Object::new();
        Reflect::set(&options, &"cols".into(), &10.into()).unwrap();
        Reflect::set(&options, &"rows".into(), &2.into()).unwrap();
        let terminal = Rc::new(xterm_web::Terminal::new(options.into()).unwrap());
        terminal.open(parent.clone().into()).unwrap();
        terminal.shared_native().write("a中bcd").unwrap();
        let renderer = terminal
            .render_service_handle()
            .unwrap()
            .current_renderer()
            .unwrap();
        let owner = Rc::clone(&terminal);
        let callback = Closure::wrap(Box::new(move || {
            if mode == "mutation" {
                owner
                    .shared_native()
                    .with_state(|state| {
                        let buffer = state.active_buffer();
                        let line = buffer.borrow().lines.get(0).unwrap();
                        line.borrow_mut().set_cell_from_codepoint(
                            3,
                            90,
                            1,
                            &AttributeData::default(),
                        );
                    })
                    .unwrap();
            } else if mode == "disposal" {
                owner.dispose().unwrap();
            }
        }) as Box<dyn Fn()>);
        let fixture = live_fixture(
            if mode == "disposal" { "error" } else { mode },
            callback.as_ref(),
        );
        let handler = field(&fixture, "handler").dyn_into::<Function>().unwrap();
        let id = terminal.register_character_joiner(handler).unwrap();
        let result = renderer.render_rows(0.0, 0.0);
        if mode == "error" || mode == "disposal" {
            assert_eq!(result.unwrap_err(), field(&fixture, "marker"), "{mode}");
        } else {
            result.unwrap();
            let row = parent
                .query_selector(".xterm-rows > :first-child")
                .unwrap()
                .unwrap();
            let text = row.text_content().unwrap();
            if mode == "unmatched" {
                assert!(text.starts_with("a中bcd"), "{text}");
            }
            if mode == "mutation" {
                assert!(text.starts_with("a中Zcd"), "{text}");
            }
            if mode == "fractional" {
                assert!(text.starts_with("a中"), "{text}");
            }
        }
        if mode != "disposal" {
            terminal.deregister_character_joiner(id).unwrap();
            terminal.dispose().unwrap();
        }
        parent.remove();
    }
}
