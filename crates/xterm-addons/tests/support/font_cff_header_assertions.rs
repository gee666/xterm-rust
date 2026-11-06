use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontLoadError, FontOptions};

#[path = "font_cff_container.rs"]
mod container;
pub use container::construct;
use container::{offset, word};

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-cff-header.json")).unwrap()
}
fn expected(case: &Value) -> Option<FontError> {
    if case["outcome"]["status"] == "ok" {
        return None;
    }
    let message = case["outcome"]["message"].as_str().unwrap();
    Some(
        if let Some(version) = message.strip_prefix("Unsupported CFF table version ") {
            let (major, minor) = version.split_once('.').unwrap();
            FontError::CffVersion {
                major: major.parse().unwrap(),
                minor: minor.parse().unwrap(),
            }
        } else if message == "Font doesn't contain TrueType, CFF or CFF2 outlines." {
            FontError::Invalid("Font doesn't contain TrueType, CFF or CFF2 outlines.")
        } else if message == "Data error" {
            FontError::Compression
        } else if message == "Cannot read properties of undefined (reading 'compression')" {
            FontError::MissingTable("name")
        } else {
            assert!(
                message == "Offset is outside the bounds of the DataView"
                    || message.starts_with("Invalid typed array length:"),
                "{message}"
            );
            FontError::Bounds
        },
    )
}
fn mutate(bytes: &mut [u8], case: &Value) {
    let woff = case["woff"] == true;
    let (start, width, count, field) = if woff {
        (44, 20, word(bytes, 12), 4)
    } else {
        (12, 16, word(bytes, 4), 8)
    };
    let at = (0..count)
        .map(|i| start + i * width)
        .filter(|at| &bytes[*at..*at + 4] == case["tag"].as_str().unwrap().as_bytes())
        .last()
        .unwrap();
    let off = offset(bytes, at + field);
    match case["mutation"].as_str().unwrap() {
        "descriptor" => {
            bytes[at + field..at + field + 4].copy_from_slice(&0xfffffff0_u32.to_be_bytes())
        }
        "major" => bytes[off] = 3,
        "repair" => bytes[off] = 1,
        "wrapper" => bytes[off] = 255,
        "throw" => {}
        value => panic!("Unknown mutation {value}"),
    }
}
pub fn assert_corpus() {
    let fixture = fixture();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let mut bytes = construct(case, &fixture);
        let hash = bytes.iter().fold(2166136261_u32, |h, b| {
            (h ^ u32::from(*b)).wrapping_mul(16777619)
        });
        assert_eq!(
            u64::from(hash),
            case["inputFnv32"].as_u64().unwrap(),
            "input {}",
            case["name"]
        );
        let mut warnings = Vec::new();
        let initial = bytes.clone();
        let actual =
            Font::load_buffer_with_refresh(&initial, FontOptions::default(), |diagnostic| {
                warnings.push(serde_json::json!(["warn", diagnostic.message()]));
                let changed = case["mutation"].is_string();
                if changed {
                    mutate(&mut bytes, case);
                }
                if case["throws"].is_string() {
                    Err(())
                } else {
                    Ok(changed.then(|| bytes.clone()))
                }
            })
            .err();
        let expected = if case["outcome"]["status"] == "warning" {
            Some(FontLoadError::Warning(()))
        } else {
            expected(case).map(FontLoadError::Font)
        };
        if actual != expected || serde_json::json!(warnings) != case["warnings"] {
            failures.push(format!(
                "{} expected {expected:?} {}, got {actual:?} {warnings:?}",
                case["name"], case["warnings"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
