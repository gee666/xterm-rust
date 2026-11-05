use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};
#[path = "font_cff_header_assertions.rs"]
mod header;

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-cff2-dict.json")).unwrap()
}
pub fn construct(case: &Value, fixture: &Value) -> Vec<u8> {
    header::construct(case, fixture)
}
pub fn assert_corpus() {
    assert_cases(|_| true);
}
pub fn assert_directed() {
    assert_cases(|case| {
        case["remove"].is_null()
            && [31, 255, 28, 30]
                .iter()
                .any(|byte| case["data"] == serde_json::json!([2, 0, 5, 0, 1, byte, 0, 0, 0, 0]))
    });
}
fn assert_cases(selected: impl Fn(&Value) -> bool) {
    let fixture = fixture();
    let mut failures = Vec::new();
    for case in fixture["cases"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|case| selected(case))
    {
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
        let expected = match case["outcome"]["message"].as_str() {
            None => None,
            Some("Invalid b0 31") => Some(FontError::CffOperand(31)),
            Some("Invalid b0 255") => Some(FontError::CffOperand(255)),
            Some("Offset is outside the bounds of the DataView") => Some(FontError::Bounds),
            Some("Cannot read properties of undefined (reading 'compression')") => {
                Some(FontError::MissingTable("name"))
            }
            value => panic!("Unmapped source outcome {value:?}"),
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
