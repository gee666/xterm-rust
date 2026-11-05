#[path = "support/font_gsub_lookup_assertions.rs"]
mod assertions;
#[path = "support/font_metadata_data.rs"]
mod corpus;

#[test]
fn type9_constructor_and_referenced_null_subtables_match_source() {
    assertions::assert_corpus();
    assertions::assert_fixture(
        &serde_json::from_str(include_str!("fixtures/font-gsub-type9.json")).unwrap(),
    );
}
