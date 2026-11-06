#![cfg(target_arch = "wasm32")]
use wasm_bindgen_test::*;
#[path = "support/font_gsub_addon_assertions.rs"]
mod addon_assertions;
#[path = "../../xterm-addons/tests/support/font_gsub_lookup_assertions.rs"]
mod assertions;
#[path = "../../xterm-addons/tests/support/font_metadata_data.rs"]
mod corpus;
wasm_bindgen_test_configure!(run_in_browser);
#[wasm_bindgen_test]
fn source_noncontext_coverage_in_wasm() {
    assertions::assert_corpus();
    assertions::assert_fixture(&fixture());
}
#[wasm_bindgen_test(async)]
async fn actual_addon_noncontext_coverage() {
    addon_assertions::assert_loading(
        fixture(),
        include_str!("fixtures/font-gsub-noncontext-addon.json"),
    )
    .await;
}
fn fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../xterm-addons/tests/fixtures/font-gsub-noncontext.json"
    ))
    .unwrap()
}
