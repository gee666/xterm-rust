#[path = "support/font_cff2_fdarray_assertions.rs"]
mod assertions;
#[test]
fn source_fdarray_guard_corpus() {
    assertions::assert_corpus();
}
#[test]
fn predecessor_dictionary_and_header_regressions() {
    assertions::assert_previous();
}
