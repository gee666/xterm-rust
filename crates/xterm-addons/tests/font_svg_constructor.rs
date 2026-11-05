#[path = "support/font_metadata_data.rs"]
mod corpus;
#[path = "support/font_svg_assertions.rs"]
mod svg;

#[test]
fn unchanged_source_svg_constructor_outcomes() {
    svg::assert_corpus();
}
