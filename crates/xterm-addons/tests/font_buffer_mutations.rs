#[path = "support/font_buffer_data.rs"]
mod corpus;

#[test]
fn source_callback_mutations_preserve_constructor_phases() {
    corpus::assert_corpus();
}
