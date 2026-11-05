#[path = "support/font_cff_header_assertions.rs"]
mod assertions;
#[test]
fn selected_cff_header_matches_unchanged_source() {
    assertions::assert_corpus();
}
