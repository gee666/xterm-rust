#[path = "support/font_cff2_dict_assertions.rs"]
mod assertions;
#[test]
fn full_source_constructor_operand_errors_and_ordering() {
    assertions::assert_corpus();
}
#[test]
fn existing_cff_header_regression() {
    assertions::assert_header_regression();
}
