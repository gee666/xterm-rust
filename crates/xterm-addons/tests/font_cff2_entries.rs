#[path = "support/font_cff2_entries_assertions.rs"]
mod assertions;
#[test]
fn full_source_constructor_duplicate_corpus() {
    assertions::assert_corpus();
}
#[test]
fn directed_duplicate_regression() {
    assertions::assert_directed();
}
#[test]
fn earlier_lexical_and_header_regressions() {
    assertions::assert_previous();
}
