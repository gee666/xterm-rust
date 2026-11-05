use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};
#[path = "font_cff_header_assertions.rs"]
mod header;

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-cff-index.json")).unwrap()
}
pub fn construct(case: &Value, fixture: &Value) -> Vec<u8> {
    let base = include_bytes!("../fixtures/ligature-fonts/FiraCode-Regular.otf");
    let word = |at| usize::from(u16::from_be_bytes(base[at..at + 2].try_into().unwrap()));
    let offset =
        |at| usize::try_from(u32::from_be_bytes(base[at..at + 4].try_into().unwrap())).unwrap();
    let at = (0..word(4))
        .map(|i| 12 + 16 * i)
        .find(|at| &base[*at..*at + 4] == b"CFF ")
        .unwrap();
    let mut data = if case["data"].is_array() {
        case["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
            .collect()
    } else {
        base[offset(at + 8)..offset(at + 8) + offset(at + 12)].to_vec()
    };
    if let Some(patches) = case["patches"].as_array() {
        for patch in patches {
            let at = usize::try_from(patch["at"].as_u64().unwrap()).unwrap();
            for (i, byte) in patch["bytes"].as_array().unwrap().iter().enumerate() {
                data[at + i] = u8::try_from(byte.as_u64().unwrap()).unwrap();
            }
        }
    }
    if let Some(length) = case["truncate"].as_u64() {
        data.truncate(usize::try_from(length).unwrap());
    }
    let mut expanded = case.clone();
    expanded["data"] = serde_json::json!(data);
    header::construct(&expanded, fixture)
}
pub fn assert_corpus() {
    let fixture = fixture();
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let bytes = construct(case, &fixture);
        let hash = bytes.iter().fold(2166136261_u32, |h, b| {
            (h ^ u32::from(*b)).wrapping_mul(16777619)
        });
        assert_eq!(
            u64::from(hash),
            case["inputFnv32"].as_u64().unwrap(),
            "input {}",
            case["name"]
        );
        let actual = Font::load_buffer(&bytes, FontOptions::default()).err();
        let expected = if case["outcome"]["status"] == "ok" {
            None
        } else {
            Some(FontError::Bounds)
        };
        if actual != expected {
            failures.push(format!(
                "{} expected {expected:?}, got {actual:?}",
                case["name"]
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
pub fn assert_header_regression() {
    header::assert_corpus();
}
