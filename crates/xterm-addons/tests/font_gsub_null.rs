#[path = "support/font_gsub_lookup_assertions.rs"]
mod assertions;
#[path = "support/font_metadata_data.rs"]
mod corpus;

#[test]
fn source_gsub_null_records_match_constructor_phases() {
    assertions::assert_corpus();
    assertions::assert_fixture(
        &serde_json::from_str(include_str!("fixtures/font-gsub-type9.json")).unwrap(),
    );
    assertions::assert_fixture(
        &serde_json::from_str(include_str!("fixtures/font-gsub-null.json")).unwrap(),
    );
}
