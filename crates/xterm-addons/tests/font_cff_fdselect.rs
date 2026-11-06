#[path = "support/font_cff_fdselect_assertions.rs"]
mod assertions;

#[test]
fn unchanged_full_source_fdselect_constructors() {
    assertions::assert_corpus();
}
#[test]
fn unchanged_predecessor_constructor_assertions() {
    assertions::assert_previous();
}
