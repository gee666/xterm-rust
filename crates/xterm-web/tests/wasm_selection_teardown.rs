#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn public_selection_target_is_written_after_drag_teardown_callbacks() -> Result<(), JsValue> {
    let program = Function::new_with_args(
        "terminal,action,phase",
        include_str!("../../../scripts/selection-teardown-trace.js"),
    );
    let mut results = Vec::new();
    for phase in ["mousemove", "mouseup"] {
        for action in [
            "observe",
            "throw",
            "nested",
            "all",
            "lines",
            "clear",
            "nested-throw",
            "all-throw",
            "lines-throw",
        ] {
            let options = Object::new();
            Reflect::set(&options, &"cols".into(), &20.into()).unwrap();
            Reflect::set(&options, &"rows".into(), &4.into()).unwrap();
            let terminal: JsValue = Terminal::new(options.into())?.into();
            let pending: Promise = program
                .call3(
                    &JsValue::UNDEFINED,
                    &terminal,
                    &action.into(),
                    &phase.into(),
                )?
                .dyn_into()?;
            let result = JsFuture::from(pending).await?;
            let actual: serde_json::Value = serde_json::from_str(
                &js_sys::JSON::stringify(&result)
                    .unwrap()
                    .as_string()
                    .unwrap(),
            )
            .unwrap();
            results.push(actual);
        }
    }
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/selection-teardown.json")).unwrap();
    assert_eq!(serde_json::Value::Array(results), expected);
    Ok(())
}
