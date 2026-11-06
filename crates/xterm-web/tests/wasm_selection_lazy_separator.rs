#![cfg(target_arch = "wasm32")]
// Copyright (c) 2017 The xterm.js authors. All rights reserved. MIT license.
use js_sys::{Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::Terminal;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test(async)]
async fn unused_separator_objects_are_not_coerced_by_click_drag_or_mouseup() -> Result<(), JsValue>
{
    let program = Function::new_with_args(
        "terminal,action,poison",
        include_str!("../../../scripts/selection-lazy-separator-trace.js"),
    );
    let mut results = Vec::new();
    for action in ["normal", "up", "shift", "triple"] {
        for poison in ["primitive-throw", "string-throw", "primitive-count"] {
            let options = Object::new();
            Reflect::set(&options, &"cols".into(), &20.into())?;
            Reflect::set(&options, &"rows".into(), &4.into())?;
            let terminal: JsValue = Terminal::new(options.into())?.into();
            let pending: Promise = program
                .call3(
                    &JsValue::UNDEFINED,
                    &terminal,
                    &action.into(),
                    &poison.into(),
                )?
                .dyn_into()?;
            let result = JsFuture::from(pending).await?;
            results.push(
                serde_json::from_str::<serde_json::Value>(
                    &js_sys::JSON::stringify(&result)?.as_string().unwrap(),
                )
                .unwrap(),
            );
        }
    }
    let expected: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/selection-lazy-separator.json")).unwrap();
    assert_eq!(serde_json::Value::Array(results), expected);
    Ok(())
}
