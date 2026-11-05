use super::corpus;
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};

pub fn assert_corpus() {
    let fixture: Value = serde_json::from_str(include_str!("../fixtures/font-svg.json")).unwrap();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let actual = Font::load_buffer(&corpus::construct(case), FontOptions::default()).err();
        let expected = match case["outcome"]["status"].as_str().unwrap() {
            "ok" => None,
            "error"
                if case["name"]
                    .as_str()
                    .unwrap()
                    .contains("reaches missing name") =>
            {
                Some(FontError::MissingTable("name"))
            }
            "error" => Some(FontError::Bounds),
            value => panic!("Unknown source status {value}"),
        };
        if actual != expected {
            failures.push(format!(
                "{}: expected {expected:?}, got {actual:?}",
                case["name"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    // Retain valid and invalid later-phase kern controls. The eager SVG hook
    // must not make the already-proven deferred constructor path unreachable.
    let metadata: Value =
        serde_json::from_str(include_str!("../fixtures/font-metadata.json")).unwrap();
    for name in [
        "kern version 0 bytes 26 SFNT",
        "kern version 2 bytes 26 SFNT",
    ] {
        let case = metadata["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["name"] == name)
            .unwrap();
        assert_eq!(
            Font::load_buffer(&corpus::construct(case), FontOptions::default()).err(),
            corpus::kern_error(case)
        );
    }
}
