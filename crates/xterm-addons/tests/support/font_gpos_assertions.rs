use super::corpus;
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontLoadError, FontOptions};

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-gpos.json")).unwrap()
}
pub fn input(fixture: &Value, case: &Value) -> Vec<u8> {
    let bytes = if case["woff"] == true {
        corpus::construct_with_woff_payloads(case, fixture.get("woffTemplate"))
    } else {
        corpus::construct(case)
    };
    let hash = bytes.iter().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    });
    assert_eq!(
        u64::from(hash),
        case["inputHash"].as_u64().unwrap(),
        "Input drift: {}",
        case["name"]
    );
    bytes
}
pub fn mutate(bytes: &mut [u8], mutation: &Value) {
    let word = |at| usize::from(u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap()));
    let woff = &bytes[..4] == b"wOFF";
    let count = word(if woff { 12 } else { 4 });
    for index in 0..count {
        let at = if woff {
            44 + index * 20
        } else {
            12 + index * 16
        };
        if &bytes[at..at + 4] == mutation["tag"].as_str().unwrap().as_bytes() {
            let at = at + if woff { 4 } else { 8 };
            let start =
                usize::try_from(u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap();
            let field = start + usize::try_from(mutation["at"].as_u64().unwrap()).unwrap();
            bytes[field..field + 2].copy_from_slice(
                &u16::try_from(mutation["value"].as_u64().unwrap())
                    .unwrap()
                    .to_be_bytes(),
            );
            return;
        }
    }
    panic!("Mutation table absent");
}
pub fn assert_corpus() {
    // Also exercises constructor_error's existing boundary mappings rather than
    // creating a second looser accepted/rejected-only comparison.
    let fixture = fixture();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let bytes = input(&fixture, case);
        let mut current = bytes.clone();
        let mut warnings = Vec::new();
        let actual = Font::load_buffer_with_refresh(&bytes, FontOptions::default(), |diagnostic| {
            warnings.push(diagnostic.message().into_owned());
            if case["throws"].is_string() {
                return Err(());
            }
            if warnings.len() == 1 && case["warningMutation"].is_object() {
                mutate(&mut current, &case["warningMutation"]);
                return Ok(Some(current.clone()));
            }
            Ok(None)
        })
        .err();
        let message = match actual {
            None => "ok".to_owned(),
            Some(FontLoadError::Warning(())) => "warning".to_owned(),
            Some(FontLoadError::Font(error)) => {
                use xterm_addons::ligatures::font::FontError;
                match error {
                    FontError::Bounds => {
                        assert_eq!(corpus::constructor_error(case), Some(FontError::Bounds));
                        "Offset is outside the bounds of the DataView".to_owned()
                    }
                    FontError::UnsupportedFormat("GSUB version", _) => {
                        "Unsupported GSUB table version.".to_owned()
                    }
                    FontError::MissingTable(_) => {
                        "Cannot read properties of undefined (reading 'compression')".to_owned()
                    }
                    error => error.to_string(),
                }
            }
        };
        let expected = case["outcome"]["message"]
            .as_str()
            .unwrap_or_else(|| case["outcome"]["status"].as_str().unwrap());
        let expected = if expected.starts_with("Invalid typed array length:") {
            "Offset is outside the bounds of the DataView"
        } else {
            expected
        };
        let expected_warnings: Vec<_> = case["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        if message != expected || warnings != expected_warnings {
            failures.push(format!(
                "{}: expected {expected:?} {expected_warnings:?}, got {message:?} {warnings:?}",
                case["name"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} GPOS mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
