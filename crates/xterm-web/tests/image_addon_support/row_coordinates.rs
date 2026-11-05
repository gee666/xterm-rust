// Copyright (c) 2020 The xterm.js authors. All rights reserved. MIT license.
use crate::support::{eval, get, promise, public_api};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

async fn compare(scenario: &str) {
    let module = JsFuture::from(js_sys::Promise::resolve(&eval("import('/suite.js')")))
        .await
        .unwrap();
    let apis = js_sys::Array::of3(
        &eval("({Terminal:SourceTerminal,ImageAddon:SourceImageAddon})"),
        &module,
        &public_api().await,
    );
    let run = js_sys::Function::new_with_args("apis,scenario", include_str!("row_coordinates.js"));
    let result = run
        .call2(&JsValue::UNDEFINED, &apis, &scenario.into())
        .unwrap();
    // The JS probe checks explicit unchanged-source outcomes before comparing
    // direct WASM and shipped constructors. Its promise rejects on any mismatch.
    promise(result).await;
    assert_eq!(get(&apis, "length"), JsValue::from(3));
}

#[wasm_bindgen_test(async)]
async fn active_row_short_circuit_and_conversion_order_match_source() {
    compare("empty").await;
}
#[wasm_bindgen_test(async)]
async fn active_raw_row_addition_matches_source_pixels_after_ring_rotation() {
    compare("raw").await;
}
#[wasm_bindgen_test(async)]
async fn active_row_backing_and_storage_survive_coordinate_reentry() {
    compare("reentry").await;
}
