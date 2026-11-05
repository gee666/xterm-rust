use super::corpus;
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-outline.json")).unwrap()
}
pub fn input(fixture: &Value, case: &Value) -> Vec<u8> {
    let mut change = case.clone();
    if let Some(length) = case["fillLength"].as_u64() {
        change["data"] = Value::Array(vec![
            case["fillValue"].clone();
            usize::try_from(length).unwrap()
        ]);
    }
    if case["woff"] == true {
        change.as_object_mut().unwrap().remove("directoryLength");
    }
    let mut bytes = corpus::construct_with_woff_payloads(&change, fixture.get("woffTemplate"));
    if case["woff"] == true {
        if let Some(length) = case["directoryLength"].as_u64() {
            let count = usize::from(u16::from_be_bytes(bytes[12..14].try_into().unwrap()));
            let at = 44 + (count - 1) * 20 + 12;
            bytes[at..at + 4].copy_from_slice(&u32::try_from(length).unwrap().to_be_bytes());
        }
    }
    let hash = bytes.iter().fold(0x811c_9dc5_u32, |hash, byte| {
        (hash ^ u32::from(*byte)).wrapping_mul(0x0100_0193)
    });
    assert_eq!(
        u64::from(hash),
        case["inputHash"].as_u64().unwrap(),
        "Input reconstruction drift: {}",
        case["name"]
    );
    bytes
}
pub fn assert_corpus() {
    let fixture = fixture();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let bytes = input(&fixture, case);
        let expected = match case["outcome"]["message"].as_str() {
            Some("Data error") => Some(FontError::Compression),
            _ => corpus::constructor_error(case),
        };
        let actual = Font::load_buffer(&bytes, FontOptions::default()).err();
        if actual != expected {
            failures.push(format!(
                "{}: expected {expected:?}, got {actual:?}",
                case["name"]
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} outline mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
