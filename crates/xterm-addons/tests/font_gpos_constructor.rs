#[path = "support/font_metadata_data.rs"]
mod corpus;
#[path = "support/font_gpos_assertions.rs"]
mod gpos;

#[test]
fn source_gpos_constructor_corpus_matches() {
    gpos::assert_corpus();
}
