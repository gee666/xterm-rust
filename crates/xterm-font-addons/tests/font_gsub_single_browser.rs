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
fn source_single_format_read_order_matches_in_wasm() {
    assertions::assert_corpus();
    assertions::assert_fixture(&single_fixture());
}
#[wasm_bindgen_test(async)]
async fn actual_addon_single_format_loading_and_cache() {
    addon_assertions::assert_loading(
        single_fixture(),
        include_str!("fixtures/font-gsub-single-addon.json"),
    )
    .await;
}

fn single_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../xterm-addons/tests/fixtures/font-gsub-single.json"
    ))
    .unwrap()
}
