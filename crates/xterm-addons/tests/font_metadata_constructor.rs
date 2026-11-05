use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};
#[path = "support/font_metadata_data.rs"]
mod corpus;
#[path = "support/font_meta_loading.rs"]
mod meta;
use corpus::construct;
#[path = "support/font_kern_warning_assertions.rs"]
mod warnings;

#[test]
fn source_kern_warning_corpus_matches_native() {
    warnings::assert_corpus();
}

#[test]
fn kern_warnings_allow_reentry_and_follow_earlier_errors() {
    warnings::assert_reentry_and_earlier_errors();
}

#[test]
fn unchanged_source_metadata_constructor_outcomes() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/font-metadata.json")).unwrap();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let actual = meta::load(case);
        let expected = case["outcome"]["status"] == "ok";
        if case["tag"] == "kern" || case["tag"] == "meta" {
            let expected_error = corpus::constructor_error(case);
            if actual.as_ref().err() != expected_error.as_ref() {
                failures.push(format!(
                    "{}: expected {expected_error:?}, got {:?}",
                    case["name"],
                    actual.err()
                ));
                continue;
            }
        }
        if actual.is_ok() != expected {
            failures.push(format!(
                "{}: expected {}, got {:?}",
                case["name"],
                case["outcome"],
                actual.err()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn native_metadata_errors_identify_failed_contract() {
    let fixture: Value = serde_json::from_str(include_str!("fixtures/font-metadata.json")).unwrap();
    for (name, expected) in [
        (
            "head wrong magic",
            FontError::Invalid("Font header has wrong magic number."),
        ),
        (
            "ltag version 2",
            FontError::Invalid("Unsupported ltag table version."),
        ),
        ("missing name", FontError::MissingTable("name")),
        ("missing hmtx", FontError::MissingTable("hmtx")),
        ("hhea EOF 35", FontError::Bounds),
        ("hmtx metrics 1 bytes 3 SFNT", FontError::Bounds),
        ("invalid last hmtx descriptor WOFF", FontError::Bounds),
        (
            "zero-read hmtx compressed invalid offset",
            FontError::Bounds,
        ),
        (
            "post version 151552 size 35",
            FontError::Type("Cannot read properties of undefined (reading '0')"),
        ),
        (
            "maxp version 0 size 6",
            FontError::Type("Cannot read properties of undefined (reading 'addUnicode')"),
        ),
    ] {
        let case = fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == name)
            .unwrap();
        assert_eq!(
            Font::load_buffer(&construct(case), FontOptions::default()).err(),
            Some(expected),
            "{name}"
        );
    }
}
