#[path = "support/font_hint_tables.rs"]
mod corpus;

#[test]
fn unchanged_source_hint_table_constructor_outcomes_and_error_order() {
    corpus::assert_corpus();
}
