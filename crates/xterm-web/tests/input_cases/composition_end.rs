//! Original CompositionHelper bodies versus Rust DOM listeners. The comparison
//! observes state during classList.remove, not only the eventual output string.
use super::support::Setup;
use js_sys::{Array, Function, Promise};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::utf16::to_js_string;

#[wasm_bindgen(module = "/tests/fixtures/composition-end-cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = compareCompositionEnd)]
    fn compare(
        area: &web_sys::HtmlTextAreaElement,
        overlay: &web_sys::HtmlElement,
        snapshot: &Function,
        bodies: &Array,
        behavior: &str,
    ) -> Result<Promise, JsValue>;
}

async fn check(behavior: &str) {
    let setup = Rc::new(Setup::new());
    let observed = setup.clone();
    let snapshot = Closure::wrap(Box::new(move || -> JsValue {
        let data = Array::new();
        for (value, user) in observed.host.data.borrow().iter() {
            data.push(&Array::of2(&to_js_string(value), &(*user).into()));
        }
        Array::of3(
            &observed.input.is_composing().into(),
            &data,
            &observed
                .dom
                .composition
                .class_list()
                .contains("active")
                .into(),
        )
        .into()
    }) as Box<dyn Fn() -> JsValue>);
    let bodies: Array = [
        include_str!("../fixtures/composition-start-body.js"),
        include_str!("../fixtures/composition-end-body.js"),
        include_str!("../fixtures/composition-finalize-body.js"),
    ]
    .into_iter()
    .map(JsValue::from_str)
    .collect();
    let pending = compare(
        &setup.dom.textarea,
        &setup.dom.composition,
        snapshot.as_ref().unchecked_ref(),
        &bodies,
        behavior,
    )
    .unwrap();
    JsFuture::from(pending).await.unwrap();
}

#[wasm_bindgen_test(async)]
async fn compositionend_observes_composing_until_dom_removal_succeeds() {
    check("plain").await;
    check("throw").await;
}

#[wasm_bindgen_test(async)]
async fn compositionend_captures_new_range_after_reentrant_dom_removal() {
    check("nested-start").await;
    check("nested-end").await;
}
