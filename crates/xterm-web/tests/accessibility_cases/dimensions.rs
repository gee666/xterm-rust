// MIT. Real DOM callbacks compare shared interventions against unchanged methods.
use crate::accessibility_support::Fixture;
use js_sys::{Reflect, JSON};
use std::rc::Rc;
use wasm_bindgen::{prelude::*, JsCast};
use wasm_bindgen_test::*;
use web_sys::HtmlElement;
use xterm_web::accessibility::AccessibilityEvent;

#[wasm_bindgen(module = "/tests/accessibility_dimensions_oracle.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = sourceDimensionsCase)]
    fn source_case(container: &HtmlElement, mode: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = rustDimensionsCase)]
    fn rust_case(
        container: &HtmlElement,
        mode: &str,
        refresh: &js_sys::Function,
        change: &js_sys::Function,
    ) -> Result<JsValue, JsValue>;
}

macro_rules! dimension_case {
    ($name:ident, $mode:literal) => {
        #[wasm_bindgen_test]
        fn $name() {
            run($mode);
        }
    };
}
dimension_case!(plain_property_assignments_match_source, "plain");
dimension_case!(style_getter_precedes_dimension_reads, "style-get");
dimension_case!(width_setter_precedes_live_row_height_reads, "width-mutate");
dimension_case!(height_setter_changes_later_row_dimensions, "height-mutate");
dimension_case!(height_setter_can_stop_the_live_row_loop, "row-stop");
dimension_case!(style_getter_error_preserves_identity, "style-throw");
dimension_case!(width_setter_error_preserves_identity, "width-throw");
dimension_case!(row_height_setter_error_preserves_identity, "row-throw");

fn run(mode: &str) {
    let fixture = Fixture::new();
    fixture.render();
    let container = fixture.manager.container();
    let expected = source_case(&container, mode).unwrap();
    let host = Rc::clone(&fixture.host);
    let refresh = Closure::<dyn Fn() -> Result<(), JsValue>>::new(move || {
        host.emit(AccessibilityEvent::Dimensions)
    });
    let host = Rc::clone(&fixture.host);
    let change = Closure::<dyn Fn(JsValue) -> Result<(), JsValue>>::new(move |delta| {
        let mut dimensions = host.dimensions.get();
        for (key, target) in [
            ("cellHeight", &mut dimensions.cell_height),
            ("canvasWidth", &mut dimensions.canvas_width),
            ("fontSize", &mut dimensions.font_size),
        ] {
            if let Some(value) = Reflect::get(&delta, &key.into())?.as_f64() {
                *target = value;
            }
        }
        host.dimensions.set(dimensions);
        if let Some(rows) = Reflect::get(&delta, &"rows".into())?.as_f64() {
            host.rows.set(rows as usize);
        }
        Ok(())
    });
    let actual = rust_case(
        &container,
        mode,
        refresh.as_ref().unchecked_ref(),
        change.as_ref().unchecked_ref(),
    )
    .unwrap();
    assert_eq!(
        JSON::stringify(&actual).unwrap().as_string(),
        JSON::stringify(&expected).unwrap().as_string(),
        "dimension callback scenario {mode}"
    );
}
