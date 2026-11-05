#[path = "support/font_avar_assertions.rs"]
mod assertions;

#[test]
fn source_avar_constructor_reads_and_warnings() {
    assertions::assert_corpus();
}
