#![cfg(target_arch = "wasm32")]
#[path = "support/input_dom.rs"]
mod support;
#[path = "input_cases/composition_geometry_terminal.rs"]
mod terminal;
use js_sys::Promise;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/fixtures/composition-geometry-cases.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = compareCompositionBounds)]
    fn compare(
        area: &web_sys::HtmlTextAreaElement,
        overlay: &web_sys::HtmlElement,
        body: &str,
        name: &str,
    ) -> Result<Promise, JsValue>;
}

const CASES: [&str; 7] = [
    "finite",
    "zero",
    "negative",
    "NaN-width",
    "NaN-height",
    "NaN-both",
    "infinite",
];

#[wasm_bindgen_test(async)]
async fn overlay_bounds_preserve_source_css_for_nan_and_finite_controls() -> Result<(), JsValue> {
    for name in CASES {
        let setup = support::Setup::new();
        let pending = compare(
            &setup.dom.textarea,
            &setup.dom.composition,
            include_str!("fixtures/composition-geometry-body.js"),
            name,
        )?;
        JsFuture::from(pending).await?;
        // Invalid measurements must not prevent the composed text from being sent.
        setup.dom.textarea.set_value("中");
        setup.dom.textarea.dispatch_event(&support::event(
            "CompositionEvent",
            "compositionend",
            &[],
        ))?;
        support::next_turn().await;
        assert_eq!(
            *setup.host.data.borrow(),
            vec![(support::units("中"), true)]
        );
    }
    Ok(())
}
