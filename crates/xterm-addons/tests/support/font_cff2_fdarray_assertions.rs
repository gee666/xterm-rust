use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};
#[path = "font_cff2_entries_assertions.rs"]
mod previous;

const MISSING: &str = "This is a CFF2 font, but FDArray information is missing";
pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-cff2-fdarray.json")).unwrap()
}
pub fn construct(case: &Value, fixture: &Value) -> Vec<u8> {
    previous::construct(case, fixture)
}
pub fn assert_corpus() {
    let fixture = fixture();
    let mut exact = 0;
    let mut presence = 0;
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
        if case["presenceOnly"] == true {
            // These source constructors reach a later FDArray INDEX error.
            // Only absence of this guard's error is covered here, not that
            // unported phase. Retain the full source outcome in the fixture.
            assert_ne!(case["outcome"]["message"], MISSING);
            assert_ne!(
                actual,
                Some(FontError::Invalid(MISSING)),
                "{}",
                case["name"]
            );
            presence += 1;
            continue;
        }
        exact += 1;
        let expected = match case["outcome"]["message"].as_str() {
            None => None,
            Some(MISSING) => Some(FontError::Invalid(MISSING)),
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
        if let Some(FontError::Invalid(MISSING)) = actual {
            assert_eq!(
                actual.unwrap().to_string(),
                case["outcome"]["message"].as_str().unwrap()
            );
        }
    }
    assert_eq!((exact, presence), (852, 8));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
pub fn assert_previous() {
    previous::assert_corpus();
    previous::assert_directed();
    previous::assert_previous();
}
