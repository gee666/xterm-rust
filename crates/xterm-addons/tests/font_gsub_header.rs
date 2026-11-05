#[path = "support/font_gsub_header_assertions.rs"]
mod assertions;
#[path = "support/font_metadata_data.rs"]
mod corpus;

#[test]
fn source_gsub_header_constructor_outcomes() {
    assertions::assert_corpus();
}
