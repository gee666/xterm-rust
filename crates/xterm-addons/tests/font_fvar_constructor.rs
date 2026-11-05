#[path = "support/font_fvar_assertions.rs"]
mod assertions;

#[test]
fn unchanged_source_fvar_constructor_outcomes_and_errors() {
    assertions::assert_corpus();
}
