#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn selection_dedup_uses_live_dimensions_after_teardown() -> Result<(), JsValue> {
    let program = Function::new_with_args(
        "terminal,action,phase",
        include_str!("../../../scripts/selection-live-geometry-trace.js"),
    );
    let mut results = Vec::new();
    for phase in ["mousemove", "mouseup"] {
        for action in ["shrink", "grow", "shrink-throw", "grow-throw"] {
            let options = Object::new();
            Reflect::set(&options, &"cols".into(), &20.into())?;
            Reflect::set(&options, &"rows".into(), &4.into())?;
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
            let json = js_sys::JSON::stringify(&result)?.as_string().unwrap();
            results.push(serde_json::from_str::<serde_json::Value>(&json).unwrap());
        }
    }
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/selection-live-geometry.json")).unwrap();
    assert_eq!(serde_json::Value::Array(results), expected);
    Ok(())
}
