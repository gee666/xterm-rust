#[path = "support/font_metadata_data.rs"]
mod corpus;
#[path = "support/font_outline_assertions.rs"]
mod outline;

#[test]
fn source_truetype_constructor_boundaries_match() {
    outline::assert_corpus();
}
