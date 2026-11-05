use super::corpus;
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-gsub-lookup.json")).unwrap()
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
fn expected_error(case: &Value) -> Option<FontError> {
    if case.get("loadOutcome").unwrap_or(&case["outcome"])["status"] == "ok" {
        return None;
    }
    let outcome = case.get("loadOutcome").unwrap_or(&case["outcome"]);
    let message = outcome["message"].as_str().unwrap();
    if outcome["name"] == "TypeError" {
        assert!(matches!(
            message,
            "Cannot convert undefined or null to object"
                | "Cannot read properties of undefined (reading 'coverage')"
        ));
        return Some(FontError::Type(if message.contains("coverage") {
            "Cannot read properties of undefined (reading 'coverage')"
        } else {
            "Cannot convert undefined or null to object"
        }));
    }
    if outcome["name"] == "RangeError" {
        assert_eq!(message, "Offset is outside the bounds of the DataView");
        return corpus::constructor_error(case);
    }
    assert_eq!(outcome["name"], "Error");
    if let Some(kind) = message
        .strip_prefix("GPOS/GSUB lookup type ")
        .and_then(|s| s.strip_suffix(" unknown."))
    {
        return Some(FontError::GposLookupType(kind.parse().unwrap()));
    }
    let error = match message {
        "GSUB Multiple Substitution Subtable identifier-format must be 1"
        | "GSUB Alternate Substitution Subtable identifier-format must be 1"
        | "GSUB ligature table identifier-format must be 1" => "GSUB noncontext substitution",
        "GSUB Extension Substitution subtable identifier-format must be 1" => "GSUB extension",
        "GSUB Reverse Chaining Contextual Single Substitution Subtable identifier-format must be 1" => "GSUB reverse chaining",
        "Unsupported META table version." => return corpus::constructor_error(case),
        _ => {
            // Source carries a byte offset in these diagnostics. The existing
            // native contract represents the format and parser family instead.
            let rest = message.strip_prefix("0x").unwrap();
            let (offset, rest) = rest.split_once(": ").unwrap();
            usize::from_str_radix(offset, 16).unwrap();
            match rest {
                "lookup type 5 format must be 1, 2 or 3." => "GSUB context",
                "lookup type 6 format must be 1, 2 or 3." => "GSUB chaining",
                _ => panic!("Unmapped source error {message}"),
            }
        }
    };
    // These witnesses point at the already-read flags word, not arbitrary data.
    let data = case["data"].as_array().unwrap();
    let format = u16::from_be_bytes([
        u8::try_from(data[16].as_u64().unwrap()).unwrap(),
        u8::try_from(data[17].as_u64().unwrap()).unwrap(),
    ]);
    Some(FontError::UnsupportedFormat(error, format))
}
pub fn assert_corpus() {
    assert_fixture(&fixture());
}
pub fn assert_fixture(fixture: &Value) {
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let actual = Font::load_buffer(&input(fixture, case), FontOptions::default()).err();
        let expected = expected_error(case);
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
