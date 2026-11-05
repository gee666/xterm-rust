#[path = "support/font_gsub_lookup_assertions.rs"]
mod assertions;
#[path = "support/font_metadata_data.rs"]
mod corpus;

#[test]
fn source_single_format_read_order_matches() {
    assertions::assert_corpus();
    assertions::assert_fixture(
        &serde_json::from_str(include_str!("fixtures/font-gsub-single.json")).unwrap(),
    );
}
