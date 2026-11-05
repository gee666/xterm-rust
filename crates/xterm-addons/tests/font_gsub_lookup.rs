#[path = "support/font_gsub_lookup_assertions.rs"]
mod assertions;
#[path = "support/font_metadata_data.rs"]
mod corpus;

#[test]
fn source_lookup_header_reads_match() {
    assertions::assert_corpus();
}
