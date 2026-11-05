use super::corpus;
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-gsub-header.json")).unwrap()
}
pub fn input(fixture: &Value, case: &Value) -> Vec<u8> {
    let bytes = if case["woff"] == true {
        corpus::construct_with_woff_payloads(case, fixture.get("woffTemplate"))
    } else {
        corpus::construct(case)
    };
    let hash = bytes.iter().fold(0x811c_9dc5_u32, |h, byte| {
        (h ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    });
    assert_eq!(
        u64::from(hash),
        case["inputHash"].as_u64().unwrap(),
        "{}",
        case["name"]
    );
    bytes
}
pub fn assert_corpus() {
    let fixture = fixture();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let actual = Font::load_buffer(&input(&fixture, case), FontOptions::default()).err();
        let expected = if case["outcome"]["status"] == "ok" {
            None
        } else {
            Some(match case["outcome"]["message"].as_str().unwrap() {
                "Unsupported GSUB table version." => {
                    FontError::Invalid("Unsupported GSUB table version.")
                }
                "GPOS/GSUB feature variations table unknown." => {
                    FontError::Invalid("GPOS/GSUB feature variations table unknown.")
                }
                // Existing shared metadata mappings cover bounds, missing-table
                // errors and the later META rejection without losing its cause.
                _ => corpus::constructor_error(case).unwrap(),
            })
        };
        if actual != expected {
            failures.push(format!(
                "{} expected {expected:?}, got {actual:?}",
                case["name"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
