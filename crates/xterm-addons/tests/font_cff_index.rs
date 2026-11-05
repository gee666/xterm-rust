#[path = "support/font_cff_index_assertions.rs"]
mod assertions;
#[test]
fn source_constructor_index_corpus_matches_native() {
    assertions::assert_corpus();
}
#[test]
fn source_header_loading_and_order_regressions_match() {
    assertions::assert_header_regression();
}
