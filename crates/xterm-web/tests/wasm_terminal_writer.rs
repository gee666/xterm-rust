#![cfg(target_arch = "wasm32")]

use js_sys::{Object, Promise, Reflect};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;
use xterm_web::Terminal;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen(module = "/tests/browser_writer_cases.js")]
extern "C" {
    #[wasm_bindgen(js_name = writerReentryCase)]
    fn reentry_case(terminal: JsValue, exceed_budget: bool, render: bool) -> Promise;
    #[wasm_bindgen(js_name = writerCompletionCase)]
    fn completion_case(terminal: JsValue) -> Promise;
    #[wasm_bindgen(js_name = writerDisposalCase)]
    fn disposal_case(terminal: JsValue) -> Promise;
}

fn terminal() -> JsValue {
    let options = Object::new();
    Reflect::set(&options, &"allowProposedApi".into(), &true.into()).unwrap();
    Terminal::new(options.into()).unwrap().into()
}

async fn assert_capture(promise: Promise, expected: serde_json::Value) {
    let actual = JsFuture::from(promise).await.unwrap();
    let json = js_sys::JSON::stringify(&actual)
        .unwrap()
        .as_string()
        .unwrap();
    let actual: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(actual, expected);
}

fn reentry_expected(exceed_budget: bool) -> serde_json::Value {
    let mut events = vec![
        serde_json::json!(["data", "\u{1b}[?1;1$y", 1]),
        serde_json::json!(["data", "\u{1b}[4;1$y", 2]),
        serde_json::json!(["cursor", 2]),
        serde_json::json!(["outer-complete", 2]),
    ];
    if exceed_budget {
        events.push(serde_json::json!(["parsed", 2]));
    }
    events.extend([
        serde_json::json!(["cursor", 3]),
        serde_json::json!(["reentry-complete", 3]),
        serde_json::json!(["parsed", 3]),
    ]);
    serde_json::json!({ "events": events, "text": "PQR" })
}

#[wasm_bindgen_test(async)]
async fn data_reentry_drains_before_parsed_when_within_source_budget() {
    assert_capture(
        reentry_case(terminal(), false, false),
        reentry_expected(false),
    )
    .await;
}

#[wasm_bindgen_test(async)]
async fn data_reentry_yields_at_source_budget_then_drains_next_batch() {
    assert_capture(
        reentry_case(terminal(), true, false),
        reentry_expected(true),
    )
    .await;
}

#[wasm_bindgen_test(async)]
async fn rendered_terminal_drains_reentry_before_first_frame_within_budget() {
    let mut expected = reentry_expected(false);
    expected["firstRenderedCursor"] = serde_json::json!(3);
    assert_capture(reentry_case(terminal(), false, true), expected).await;
}

#[wasm_bindgen_test(async)]
async fn completion_callback_can_enqueue_another_write() {
    assert_capture(
        completion_case(terminal()),
        serde_json::json!({
            "events": [["first", 1], ["second", 2], ["parsed", 2]],
            "text": "AB"
        }),
    )
    .await;
}

#[wasm_bindgen_test(async)]
async fn dispose_cancels_queued_write_and_completion() {
    assert_capture(disposal_case(terminal()), serde_json::json!([])).await;
}
