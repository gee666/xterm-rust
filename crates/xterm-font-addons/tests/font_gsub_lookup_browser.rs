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
fn source_gsub_lookup_corpus_matches_in_wasm() {
    assertions::assert_corpus();
}
#[wasm_bindgen_test(async)]
async fn actual_addon_loading_failure_and_cache() {
    addon_assertions::assert_loading(
        assertions::fixture(),
        include_str!("fixtures/font-gsub-lookup-addon.json"),
    )
    .await;
}
