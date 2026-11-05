use super::corpus;
use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontLoadError, FontOptions};

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-gdef.json")).unwrap()
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
        "Input reconstruction drift: {}",
        case["name"]
    );
    bytes
}
fn word(bytes: &[u8], at: usize) -> usize {
    usize::from(u16::from_be_bytes(bytes[at..at + 2].try_into().unwrap()))
}
fn offset(bytes: &[u8], at: usize) -> usize {
    usize::try_from(u32::from_be_bytes(bytes[at..at + 4].try_into().unwrap())).unwrap()
}
pub fn mutate(bytes: &mut [u8], mutation: &Value) {
    let compressed = &bytes[..4] == b"wOFF";
    let (start, width, count) = if compressed {
        (44, 20, 12)
    } else {
        (12, 16, 4)
    };
    for index in 0..word(bytes, count) {
        let at = start + index * width;
        if &bytes[at..at + 4] == mutation["tag"].as_str().unwrap().as_bytes() {
            let field = offset(bytes, at + if compressed { 4 } else { 8 })
                + usize::try_from(mutation["at"].as_u64().unwrap()).unwrap();
            bytes[field..field + 2].copy_from_slice(
                &u16::try_from(mutation["value"].as_u64().unwrap())
                    .unwrap()
                    .to_be_bytes(),
            );
            return;
        }
    }
    panic!("Mutation target missing");
}
fn error(case: &Value) -> Option<FontLoadError<()>> {
    match case["outcome"]["status"].as_str().unwrap() {
        "ok" => None,
        "warning" => Some(FontLoadError::Warning(())),
        "error" => {
            let message = case["outcome"]["message"].as_str().unwrap();
            let error = match message {
                "Data error" => FontError::Compression,
                "Unsupported fvar table version." => {
                    FontError::Invalid("Unsupported fvar table version.")
                }
                "Unsupported GDEF table version." => {
                    FontError::Invalid("Unsupported GDEF table version.")
                }
                "Unsupported CaretValue table version." => {
                    FontError::Invalid("Unsupported CaretValue table version.")
                }
                text if text.ends_with(": Coverage format must be 1 or 2.") => {
                    FontError::CoverageFormat {
                        offset: usize::from_str_radix(
                            text.split(':').next().unwrap().trim_start_matches("0x"),
                            16,
                        )
                        .unwrap(),
                    }
                }
                _ => corpus::constructor_error(case).unwrap(),
            };
            Some(FontLoadError::Font(error))
        }
        value => panic!("Unexpected source status {value}"),
    }
}
pub fn assert_corpus() {
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
        let expected = error(case);
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
        "{} GDEF mismatches:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
