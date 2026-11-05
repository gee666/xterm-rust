#![cfg(target_arch = "wasm32")]
//! Source-derived operation tests, not proof of live Scrollable/Element parity.
use js_sys::{Array, Function};
use wasm_bindgen::{closure::Closure, prelude::*, JsCast};
use wasm_bindgen_test::*;
use xterm_web::scroll::{animation::SmoothOperation, state::ScrollState};
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/raw_scroll_animation_oracle.js")]
extern "C" {
    #[wasm_bindgen(catch, js_name = runAnimationPrimitives)]
    fn primitives(run: &Function) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = runAnimationObjects)]
    fn objects(run: &Function) -> Result<u32, JsValue>;
    #[wasm_bindgen(catch, js_name = runAnimationDimensions)]
    fn dimensions(run: &Function) -> Result<u32, JsValue>;
}

fn state(fields: Array) -> Result<ScrollState<JsValue>, JsValue> {
    let mut state = ScrollState::try_new(
        false,
        100.0.into(),
        10000.0.into(),
        0.0.into(),
        100.0.into(),
        10000.0.into(),
        0.0.into(),
    )?;
    // The source operation takes ISmoothScrollPosition, not only ScrollState.
    state.left = fields.get(0);
    state.top = fields.get(1);
    state.width = fields.get(2);
    state.height = fields.get(3);
    Ok(state)
}
fn row(values: &[JsValue]) -> JsValue {
    values.iter().collect::<Array>().into()
}
fn record(result: Result<JsValue, JsValue>) -> JsValue {
    match result {
        Ok(value) => row(&[true.into(), value]),
        Err(error) => row(&[false.into(), error]),
    }
}
fn run_operation(
    from: Array,
    to: Array,
    timing: Array,
    actions: Array,
) -> Result<JsValue, JsValue> {
    let from = state(from)?;
    let to = state(to)?;
    let mut operation = if timing.get(0).is_truthy() {
        SmoothOperation::try_start(from, to, timing.get(2), || timing.get(1).as_f64().unwrap())?
    } else {
        SmoothOperation::try_new(from, to, timing.get(1), timing.get(2))?
    };
    let output = Array::new();
    output.push(&row(&[
        operation.start_time.clone(),
        operation.duration.clone(),
    ]));
    for action in actions.iter() {
        let action = Array::from(&action);
        let result = match action.get(0).as_string().as_deref() {
            Some("hook") => {
                action
                    .get(1)
                    .unchecked_into::<Function>()
                    .call0(&JsValue::UNDEFINED)?;
                continue;
            }
            Some("accept") => {
                let args = Array::from(&action.get(1));
                let next = ScrollState::try_new(
                    false,
                    args.get(0),
                    args.get(1),
                    args.get(2),
                    args.get(3),
                    args.get(4),
                    args.get(5),
                );
                next.and_then(|state| operation.try_accept_dimensions(state))
                    .map(|()| JsValue::UNDEFINED)
            }
            Some("tick") => operation
                .try_tick(action.get(1).as_f64().unwrap())
                .map(|update| {
                    row(&[
                        update.position.left,
                        update.position.top,
                        update.done.into(),
                    ])
                }),
            _ => return Err(js_sys::Error::new("unknown test action").into()),
        };
        output.push(&row(&[
            record(result),
            operation.to.left.clone(),
            operation.to.top.clone(),
            operation.to.width.clone(),
            operation.to.height.clone(),
        ]));
    }
    Ok(output.into())
}
fn check(check: fn(&Function) -> Result<u32, JsValue>, count: u32) -> Result<(), JsValue> {
    let run = Closure::wrap(Box::new(run_operation)
        as Box<dyn Fn(Array, Array, Array, Array) -> Result<JsValue, JsValue>>);
    assert_eq!(check(run.as_ref().unchecked_ref())?, count);
    Ok(())
}
#[wasm_bindgen_test]
fn source_primitive_paths_and_raw_time_values() -> Result<(), JsValue> {
    check(primitives, 270)
}
#[wasm_bindgen_test]
fn source_cached_deltas_identity_coercions_and_errors() -> Result<(), JsValue> {
    check(objects, 145)
}
#[wasm_bindgen_test]
fn source_dimension_acceptance_and_partial_failure_assignments() -> Result<(), JsValue> {
    check(dimensions, 125)
}
