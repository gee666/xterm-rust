#![cfg(target_arch = "wasm32")]
//! Direct differential tests of shared Rust state against unchanged source.
//! The browser Scrollable/Element still need migration to these raw values.
use js_sys::Array;
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::scroll::state::ScrollState;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/raw_scroll_state_oracle.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runRawStatePrimitives)]
    fn primitives(run: &js_sys::Function) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = runRawStateObjects)]
    fn objects(run: &js_sys::Function) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = runRawStateUpdates)]
    fn updates(run: &js_sys::Function) -> Result<u32, JsValue>;
}
fn snapshot(
    integer: bool,
    args: Array,
    action: JsValue,
    update: JsValue,
    raw: bool,
) -> Result<JsValue, JsValue> {
    let previous = ScrollState::<JsValue>::try_new(
        integer,
        args.get(0),
        args.get(1),
        args.get(2),
        args.get(3),
        args.get(4),
        args.get(5),
    )?;
    let state = match action.as_string().as_deref() {
        Some("position") => previous.try_with_raw_position(&update)?,
        Some("dimensions") => previous.try_with_raw_dimensions(&update, raw)?,
        _ => previous.clone(),
    };
    let event = state.event(previous, false);
    let values = Array::new();
    for value in [
        &state.raw_left,
        &state.raw_top,
        &state.width,
        &state.scroll_width,
        &state.left,
        &state.height,
        &state.scroll_height,
        &state.top,
    ] {
        values.push(value);
    }
    for value in [
        event.changed.width,
        event.changed.scroll_width,
        event.changed.left,
        event.changed.height,
        event.changed.scroll_height,
        event.changed.top,
        state.equals(&state),
    ] {
        values.push(&value.into());
    }
    Ok(values.into())
}

fn run(check: fn(&js_sys::Function) -> Result<u32, JsValue>, count: u32) -> Result<(), JsValue> {
    let callback = Closure::wrap(Box::new(snapshot)
        as Box<dyn Fn(bool, Array, JsValue, JsValue, bool) -> Result<JsValue, JsValue>>);
    assert_eq!(check(callback.as_ref().unchecked_ref())?, count);
    Ok(())
}
#[wasm_bindgen_test]
fn source_primitive_matrix_preserves_values_and_failures() -> Result<(), JsValue> {
    run(primitives, 180)
}
#[wasm_bindgen_test]
fn source_object_identity_coercion_order_and_errors() -> Result<(), JsValue> {
    run(objects, 27)
}
#[wasm_bindgen_test]
fn source_inherited_getters_repeated_reads_and_raw_updates() -> Result<(), JsValue> {
    run(updates, 80)
}
