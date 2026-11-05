#[path = "font_metadata_data.rs"]
mod corpus;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};

pub fn assert_corpus() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/font-fvar.json")).unwrap();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let expected = if case["outcome"]["message"] == "Unsupported fvar table version." {
            Some(FontError::Invalid("Unsupported fvar table version."))
        } else {
            corpus::constructor_error(case)
        };
        let actual = Font::load_buffer(&corpus::construct(case), FontOptions::default()).err();
        if actual != expected {
            failures.push(format!(
                "{}: expected {expected:?}, got {actual:?}",
                case["name"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
