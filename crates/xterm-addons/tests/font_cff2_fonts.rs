#[path = "support/font_cff2_fonts_assertions.rs"]
mod assertions;

#[test]
fn complete_source_cff2_font_and_private_constructors() {
    assertions::assert_corpus();
}
#[test]
fn predecessor_cff_constructor_assertions_remain_intact() {
    assertions::assert_previous();
}
