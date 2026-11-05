#[path = "support/font_metadata_data.rs"]
mod corpus;
#[path = "support/font_gdef_assertions.rs"]
mod gdef;

#[test]
fn unchanged_source_gdef_constructor_outcomes() {
    gdef::assert_corpus();
}
