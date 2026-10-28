// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Reentrant failure recovery through the actual exported Terminal and DOM.
use super::{units, Opened};
use js_sys::{Function, Object, Reflect};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::BrowserRuntime;
#[path = "../accessibility_support/errors.rs"]
mod errors;
#[wasm_bindgen(module = "/tests/accessibility_support/reentry.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = observeResetRender)]
    fn observe_reset_render(
        root: &HtmlElement,
        callback: &Function,
    ) -> Result<js_sys::Promise, JsValue>;
    #[wasm_bindgen(catch, js_name = resizeOnComparison)]
    fn resize_on_comparison(node: &web_sys::Node, callback: &Function)
        -> Result<Function, JsValue>;
    #[wasm_bindgen(catch, js_name = failRowCreation)]
    fn fail_row_creation(
        document: &web_sys::Document,
        last: &HtmlElement,
        marker: &JsValue,
    ) -> Result<Function, JsValue>;
    #[wasm_bindgen(catch, js_name = setTextValue)]
    fn set_text_value(row: &web_sys::Node, value: &JsValue) -> Result<Function, JsValue>;
}

#[wasm_bindgen_test(async)]
async fn reset_from_first_row_setter_keeps_original_buffer_for_the_whole_render() {
    let f = Opened::new(&BrowserRuntime::new()).unwrap();
    f.write(&units("N0\r\nN1\r\nN2")).await;
    f.expect(&f.row(), &units("N0")).await;
    let terminal = Rc::clone(&f.terminal);
    let reset = Closure::<dyn FnMut()>::new(move || terminal.reset().unwrap());
    let snapshot = observe_reset_render(
        &f.terminal.element().unwrap(),
        reset.as_ref().unchecked_ref(),
    )
    .unwrap();
    f.terminal.refresh(0.0, 2.0).unwrap();
    let snapshot = js_sys::Array::from(&JsFuture::from(snapshot).await.unwrap());
    assert_eq!(snapshot.length(), 3);
    for (index, expected) in ["N0", "N1", "N2"].into_iter().enumerate() {
        assert_eq!(
            snapshot.get(index as u32).as_string().as_deref(),
            Some(expected)
        );
    }
}

#[wasm_bindgen_test(async)]
async fn selection_clamps_to_live_last_row_after_comparison_resizes_terminal() {
    let f = Opened::new(&BrowserRuntime::new()).unwrap();
    f.write(&units("N0\r\nN1\r\nN2")).await;
    f.expect(&f.row(), &units("N0")).await;
    let document = web_sys::window().unwrap().document().unwrap();
    let after = document.create_element("p").unwrap();
    after.set_text_content(Some("outside"));
    document.body().unwrap().append_child(&after).unwrap();
    let anchor = f.row().first_child().unwrap();
    let focus = after.first_child().unwrap();
    let selection = document.get_selection().unwrap().unwrap();
    selection
        .set_base_and_extent(&anchor, 0, &focus, 7)
        .unwrap();
    let terminal = Rc::clone(&f.terminal);
    let resize = Closure::<dyn FnMut()>::new(move || terminal.resize(8.0, 2.0).unwrap());
    let cleanup = resize_on_comparison(&anchor, resize.as_ref().unchecked_ref()).unwrap();
    document
        .dispatch_event(&web_sys::Event::new("selectionchange").unwrap())
        .unwrap();
    cleanup.call0(&JsValue::UNDEFINED).unwrap();
    assert_eq!(f.terminal.rows().unwrap(), 2.0);
    assert_eq!(
        f.terminal.get_selection().unwrap().as_string().as_deref(),
        Some("N0\r\nN1")
    );
    selection.remove_all_ranges().unwrap();
    after.remove();
}

#[wasm_bindgen_test]
fn failed_row_creation_during_resize_preserves_listeners_and_recovers() {
    let f = Opened::new(&BrowserRuntime::new()).unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let root = f.terminal.element().unwrap();
    let last = root
        .query_selector_all(".xterm-accessibility-tree > div")
        .unwrap()
        .item(2)
        .unwrap()
        .unchecked_into::<HtmlElement>();
    let marker = Object::new();
    let cleanup = fail_row_creation(&document, &last, &marker).unwrap();
    let failure = f.terminal.resize(8.0, 4.0).unwrap_err();
    cleanup.call0(&JsValue::UNDEFINED).unwrap();
    assert_eq!(failure, JsValue::from(marker));
    f.terminal.resize(8.0, 5.0).unwrap();
    assert_eq!(
        root.query_selector_all(".xterm-accessibility-tree > div")
            .unwrap()
            .length(),
        5
    );
}

#[wasm_bindgen_test(async)]
async fn clamped_selection_uses_optional_raw_text_length_and_preserves_getter_errors() {
    let f = Opened::new(&BrowserRuntime::new()).unwrap();
    f.write(&units("N0\r\nN1\r\nN2")).await;
    f.expect(&f.row(), &units("N0")).await;
    let document = web_sys::window().unwrap().document().unwrap();
    let last = f
        .terminal
        .element()
        .unwrap()
        .query_selector_all(".xterm-accessibility-tree > div")
        .unwrap()
        .item(2)
        .unwrap();
    let after = document.create_element("p").unwrap();
    after.set_text_content(Some("outside"));
    document.body().unwrap().append_child(&after).unwrap();
    let anchor = f.row().first_child().unwrap();
    let focus = after.first_child().unwrap();
    let selection = document.get_selection().unwrap().unwrap();
    for value in [
        JsValue::NULL,
        JsValue::UNDEFINED,
        JsValue::TRUE,
        JsValue::from_f64(3.0),
        JsValue::from_str(""),
        js_sys::Symbol::for_("text").into(),
        Function::new_no_args("return {length:'0'};")
            .call0(&JsValue::UNDEFINED)
            .unwrap(),
    ] {
        f.terminal.clear_selection().unwrap();
        selection
            .set_base_and_extent(&anchor, 0, &focus, 7)
            .unwrap();
        let cleanup = set_text_value(&last, &value).unwrap();
        let error = errors::selection_error(&document).unwrap();
        cleanup.call0(&JsValue::UNDEFINED).unwrap();
        assert!(
            error.is_undefined(),
            "optional text length must not throw: {error:?}"
        );
        let range = f.terminal.get_selection_position().unwrap();
        // Source SelectionModel.finalSelectionEnd represents an exact right
        // edge as [cols, previous row], deliberately excluding the trailing EOL.
        for (name, expected_x, expected_y) in [("start", 0.0, 0.0), ("end", 8.0, 1.0)] {
            let point = Reflect::get(&range, &name.into()).unwrap();
            assert_eq!(
                Reflect::get(&point, &"x".into()).unwrap().as_f64(),
                Some(expected_x)
            );
            assert_eq!(
                Reflect::get(&point, &"y".into()).unwrap().as_f64(),
                Some(expected_y)
            );
        }
        assert_eq!(
            f.terminal.get_selection().unwrap().as_string().as_deref(),
            Some("N0\r\nN1")
        );
    }
    let marker = Object::new();
    for expression in [
        "return {get length(){throw marker;}}",
        "return {length:{valueOf(){throw marker;}}}",
    ] {
        let value = Function::new_with_args("marker", expression)
            .call1(&JsValue::UNDEFINED, &marker)
            .unwrap();
        selection
            .set_base_and_extent(&anchor, 0, &focus, 7)
            .unwrap();
        let cleanup = set_text_value(&last, &value).unwrap();
        let error = errors::selection_error(&document).unwrap();
        cleanup.call0(&JsValue::UNDEFINED).unwrap();
        assert_eq!(error, JsValue::from(marker.clone()));
    }
    selection.remove_all_ranges().unwrap();
    after.remove();
}
