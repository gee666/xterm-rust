#[path = "support/font_color_assertions.rs"]
mod assertions;
#[test]
fn eager_color_tables_match_unchanged_source_reads_warnings_and_mutations() {
    assertions::assert_corpus();
}
