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
    let mut bytes = if case["woff"] == true {
        corpus::construct_with_woff_payloads(&change, fixture.get("woffTemplate"))
    } else {
        corpus::construct(&change)
    };
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
fn mutate(bytes: &mut [u8], mutation: &Value) {
    let woff = &bytes[..4] == b"wOFF";
    let (start, width, count_at) = if woff { (44, 20, 12) } else { (12, 16, 4) };
    let count = usize::from(u16::from_be_bytes(
        bytes[count_at..count_at + 2].try_into().unwrap(),
    ));
    for index in 0..count {
        let at = start + index * width;
        if &bytes[at..at + 4] != mutation["tag"].as_str().unwrap().as_bytes() {
            continue;
        }
        let offset_at = at + if woff { 4 } else { 8 };
        let offset = if mutation["descriptor"] == true {
            offset_at
        } else {
            usize::try_from(u32::from_be_bytes(
                bytes[offset_at..offset_at + 4].try_into().unwrap(),
            ))
            .unwrap()
                + usize::try_from(mutation["at"].as_u64().unwrap()).unwrap()
        };
        bytes[offset..offset + 2].copy_from_slice(
            &u16::try_from(mutation["value"].as_u64().unwrap())
                .unwrap()
                .to_be_bytes(),
        );
        return;
    }
    panic!("Mutation target missing");
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
        let mut current = bytes.clone();
        let mut warnings = Vec::new();
        let actual = Font::load_buffer_with_refresh(&bytes, FontOptions::default(), |warning| {
            warnings.push(warning.message().into_owned());
            if warnings.len() == 1 && case["warningMutation"].is_object() {
                mutate(&mut current, &case["warningMutation"]);
                return Ok::<_, ()>(Some(current.clone()));
            }
            Ok(None)
        })
        .err()
        .map(|error| match error {
            xterm_addons::ligatures::font::FontLoadError::Font(error) => error,
            xterm_addons::ligatures::font::FontLoadError::Warning(()) => {
                panic!("No throwing callback")
            }
        });
        let expected_warnings: Vec<_> = case["warnings"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap())
            .collect();
        if actual != expected || warnings != expected_warnings {
            failures.push(format!(
                "{}: expected {expected:?} {expected_warnings:?}, got {actual:?} {warnings:?}",
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
