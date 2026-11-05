#[path = "support/font_gsub_lookup_assertions.rs"]
mod assertions;
#[path = "support/font_metadata_data.rs"]
mod corpus;
#[test]
fn source_null_gsub_subtables() {
    assertions::assert_corpus();
    for fixture in [
        include_str!("fixtures/font-gsub-type9.json"),
        include_str!("fixtures/font-gsub-null.json"),
        include_str!("fixtures/font-gsub-refs.json"),
        include_str!("fixtures/font-gsub-subtables.json"),
    ] {
        assertions::assert_fixture(&serde_json::from_str(fixture).unwrap());
    }
}
