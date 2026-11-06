#[path = "support/font_cff2_index_assertions.rs"]
mod assertions;
#[test]
fn source_complete_fdarray_index_rejections() {
    assertions::assert_corpus();
}
#[test]
fn retained_dictionary_and_header_assertions() {
    assertions::assert_previous();
}
