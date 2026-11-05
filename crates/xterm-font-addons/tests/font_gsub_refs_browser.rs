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
fn source_out_of_range_references_in_wasm() {
    assertions::assert_corpus();
    for fixture in [
        include_str!("../../xterm-addons/tests/fixtures/font-gsub-type9.json"),
        include_str!("../../xterm-addons/tests/fixtures/font-gsub-null.json"),
    ] {
        assertions::assert_fixture(&serde_json::from_str(fixture).unwrap());
    }
    assertions::assert_fixture(&refs_fixture());
}
#[wasm_bindgen_test(async)]
async fn actual_addon_out_of_range_references() {
    addon_assertions::assert_loading(
        refs_fixture(),
        include_str!("fixtures/font-gsub-refs-addon.json"),
    )
    .await;
}
fn refs_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../xterm-addons/tests/fixtures/font-gsub-refs.json"
    ))
    .unwrap()
}
