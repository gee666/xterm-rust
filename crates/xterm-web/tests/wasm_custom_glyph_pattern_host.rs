// Copyright (c) 2026 xterm-rust contributors. MIT license.
#![cfg(target_arch = "wasm32")]
use serde_json::Value;
use wasm_bindgen::prelude::*;
use wasm_bindgen_test::*;
use web_sys::CanvasRenderingContext2d;
use xterm_web::webgl::custom_glyphs::try_draw_custom_glyph;
#[path = "support/custom_glyph_canvas.rs"]
mod canvas;
wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/reference/custom-glyph-pattern-host.mjs")]
extern "C" {
    #[wasm_bindgen(catch, js_name = sourcePatternHost)]
    fn source(ctx: &CanvasRenderingContext2d, variant: i32) -> Result<bool, JsValue>;
    #[wasm_bindgen(catch, js_name = installPatternHostProbe)]
    fn install(
        ctx: &CanvasRenderingContext2d,
        kind: &str,
        color: &str,
        token: &JsValue,
    ) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = finishPatternHostProbe)]
    fn finish(probe: &JsValue, ok: bool, result: bool, error: &JsValue) -> Result<String, JsValue>;
}
const METRICS: [f64; 8] = [1.0, 1.0, 8.0, 16.0, 8.0, 16.0, 15.0, 1.0];

fn capture(
    kind: &str,
    color: &str,
    variant: i32,
    original: bool,
    token: &JsValue,
) -> (Value, CanvasRenderingContext2d) {
    let ctx = canvas::context();
    let log = canvas::log();
    let probe = install(&ctx, kind, color, token).unwrap();
    let result = if original {
        source(&ctx, variant)
    } else {
        try_draw_custom_glyph(&ctx, &[0x2591], canvas::params(&METRICS, variant), &log)
    };
    // Both results are captured and globals restored before any assertion fails.
    let json = match result {
        Ok(value) => finish(&probe, true, value, &JsValue::UNDEFINED),
        Err(error) => finish(&probe, false, false, &error),
    }
    .unwrap();
    (serde_json::from_str(&json).unwrap(), ctx)
}
fn pair(kind: &str, color: &str, variant: i32) -> (Value, Value) {
    let token: JsValue = js_sys::Symbol::for_("custom pattern host failure").into();
    let (mut expected, a) = capture(kind, color, variant, true, &token);
    let (mut actual, b) = capture(kind, color, variant, false, &token);
    let pixels =
        |ctx: &CanvasRenderingContext2d| ctx.get_image_data(0.0, 0.0, 48.0, 48.0).unwrap().data().0;
    expected["pixelsMatchSource"] = Value::Bool(true);
    actual["pixelsMatchSource"] = Value::Bool(pixels(&b) == pixels(&a));
    (expected, actual)
}
fn retain_mismatch(label: &str, expected: Value, actual: Value, failures: &mut Vec<String>) {
    if expected != actual {
        failures.push(format!("{label}: expected {expected}, actual {actual}"));
    }
}
fn assert_all(failures: Vec<String>) {
    for failure in &failures {
        console_log!("{}", failure);
    }
    assert!(
        failures.is_empty(),
        "{} host phase mismatches",
        failures.len()
    );
}

#[wasm_bindgen_test]
fn pattern_factory_arguments_and_live_data_reads_match_source() {
    let mut failures = Vec::new();
    for (variant, color) in [(0, "#193d71"), (3, "#193d72")] {
        let (expected, actual) = pair("trace", color, variant);
        assert_eq!(expected["outcome"], serde_json::json!(["ok", true]));
        retain_mismatch(color, expected, actual, &mut failures);
    }
    assert_all(failures);
}

#[wasm_bindgen_test]
fn pattern_constructor_data_and_callback_errors_keep_source_phase_and_identity() {
    let mut failures = Vec::new();
    for (kind, color) in [
        ("canvas-throw", "#194e71"),
        ("image-throw", "#194e72"),
        ("unsupported-color", "#194e73"),
        ("data-throw", "#194e74"),
        ("pattern-throw", "#194e75"),
    ] {
        let (expected, actual) = pair(kind, color, 3);
        if kind != "unsupported-color" {
            assert_eq!(expected["outcome"], serde_json::json!(["error", "token"]));
        }
        retain_mismatch(kind, expected, actual, &mut failures);
    }
    // A failed data getter must not publish a cached pattern into later calls.
    let (expected, actual) = pair("trace", "#194e74", 3);
    assert_eq!(expected["outcome"], serde_json::json!(["ok", true]));
    retain_mismatch(
        "retry after failed data getter",
        expected,
        actual,
        &mut failures,
    );
    assert_all(failures);
}

#[wasm_bindgen_test]
fn pattern_factory_availability_is_captured_like_source_module() {
    let (expected, actual) = pair("offscreen-toggle", "#196f71", 0);
    if expected["factoryWasAvailable"] == Value::Bool(true) {
        assert_eq!(expected["outcome"][0], "error");
        assert_eq!(expected["outcome"][1], "TypeError");
    } else {
        assert_eq!(expected["outcome"], serde_json::json!(["ok", true]));
    }
    assert_eq!(actual, expected);
}

#[wasm_bindgen_test]
fn pattern_transform_getter_precedes_matrix_construction() {
    let mut failures = Vec::new();
    for (variant, color) in [(0, "#197a71"), (3, "#197a72")] {
        let (expected, actual) = pair("transform-getter-throw", color, variant);
        assert_eq!(expected["outcome"], serde_json::json!(["error", "token"]));
        retain_mismatch(color, expected, actual, &mut failures);
    }
    assert_all(failures);
}

#[wasm_bindgen_test]
fn pattern_zero_variant_does_not_call_matrix_translate() {
    let mut failures = Vec::new();
    for (variant, color) in [
        (0, "#195f71"),
        (1, "#195f72"),
        (2, "#195f73"),
        (3, "#195f74"),
    ] {
        let (expected, actual) = pair("translate-throw", color, variant);
        let expected_outcome = if variant == 0 {
            serde_json::json!(["ok", true])
        } else {
            serde_json::json!(["error", "token"])
        };
        assert_eq!(expected["outcome"], expected_outcome);
        retain_mismatch(color, expected, actual, &mut failures);
    }
    assert_all(failures);
}
