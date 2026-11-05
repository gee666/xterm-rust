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
fn source_gsub_null_records_match_in_wasm() {
    assertions::assert_corpus();
    assertions::assert_fixture(
        &serde_json::from_str(include_str!(
            "../../xterm-addons/tests/fixtures/font-gsub-type9.json"
        ))
        .unwrap(),
    );
    assertions::assert_fixture(&null_fixture());
}
#[wasm_bindgen_test(async)]
async fn actual_addon_null_record_loading_and_cache() {
    addon_assertions::assert_loading(
        null_fixture(),
        include_str!("fixtures/font-gsub-null-addon.json"),
    )
    .await;
}

fn null_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../xterm-addons/tests/fixtures/font-gsub-null.json"
    ))
    .unwrap()
}
