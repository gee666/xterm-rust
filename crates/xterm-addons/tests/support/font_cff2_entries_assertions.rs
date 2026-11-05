use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};
#[path = "font_cff2_dict_assertions.rs"]
mod previous;

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-cff2-entries.json")).unwrap()
}
pub fn construct(case: &Value, fixture: &Value) -> Vec<u8> {
    previous::construct(case, fixture)
}
pub fn assert_corpus() {
    assert_cases(false);
}
pub fn assert_directed() {
    assert_cases(true);
}
fn assert_cases(directed: bool) {
    let fixture = fixture();
    let mut failures = Vec::new();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap() {
        if directed
            && (case["woff"] == true
                || !case["outcome"]["message"]
                    .as_str()
                    .is_some_and(|m| m.starts_with("Object ")))
        {
            continue;
        }
        if directed && count >= 16 {
            break;
        }
        count += 1;
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
            Some(message) if message.starts_with("Object [object Object] already has key ") => {
                Some(FontError::CffDuplicate(
                    message.rsplit(' ').next().unwrap().parse().unwrap(),
                ))
            }
            value => panic!("Unmapped source outcome {value:?}"),
        };
        if actual != expected {
            failures.push(format!(
                "{} expected {expected:?}, got {actual:?}",
                case["name"]
            ));
        }
        if let Some(FontError::CffDuplicate(_)) = actual {
            assert_eq!(
                actual.unwrap().to_string(),
                case["outcome"]["message"].as_str().unwrap()
            );
        }
    }
    assert_eq!(count, if directed { 16 } else { 898 });
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
pub fn assert_previous() {
    previous::assert_corpus();
    previous::assert_directed();
    previous::assert_header_regression();
}
