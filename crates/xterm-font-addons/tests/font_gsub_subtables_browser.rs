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
fn source_null_subtables_in_wasm() {
    assertions::assert_corpus();
    for fixture in [
        include_str!("../../xterm-addons/tests/fixtures/font-gsub-type9.json"),
        include_str!("../../xterm-addons/tests/fixtures/font-gsub-null.json"),
        include_str!("../../xterm-addons/tests/fixtures/font-gsub-refs.json"),
    ] {
        assertions::assert_fixture(&serde_json::from_str(fixture).unwrap());
    }
    assertions::assert_fixture(&subtables_fixture());
}
#[wasm_bindgen_test(async)]
async fn actual_addon_null_subtables() {
    addon_assertions::assert_loading(
        subtables_fixture(),
        include_str!("fixtures/font-gsub-subtables-addon.json"),
    )
    .await;
}
fn subtables_fixture() -> serde_json::Value {
    serde_json::from_str(include_str!(
        "../../xterm-addons/tests/fixtures/font-gsub-subtables.json"
    ))
    .unwrap()
}
