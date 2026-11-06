use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};
#[path = "font_cff2_fdarray_assertions.rs"]
mod previous;
pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-cff2-index.json")).unwrap()
}
pub fn construct(case: &Value, fixture: &Value) -> Vec<u8> {
    previous::construct(case, fixture)
}
pub fn assert_corpus() {
    let fixture = fixture();
    let mut exact = 0;
    let mut observations = 0;
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let bytes = construct(case, &fixture);
        assert_eq!(
            u64::from(bytes.iter().fold(2166136261_u32, |h, b| {
                (h ^ u32::from(*b)).wrapping_mul(16777619)
            })),
            case["inputFnv32"].as_u64().unwrap(),
            "input {}",
            case["name"]
        );
        let actual = Font::load_buffer(&bytes, FontOptions::default()).err();
        let phase = &case["phaseWhole"];
        if phase["status"] != "error" && case["data"].is_array() {
            // Complete outcomes and source resource failures are retained.
            // Successful INDEX traversal does not establish later dictionary,
            // FDSelect or charstring equivalence. Private unit tests compare
            // table-only INDEX phase outcomes, including successful traversal.
            observations += 1;
            continue;
        }
        exact += 1;
        let expected = match case["outcome"]["message"].as_str() {
            None => None,
            Some("Offset is outside the bounds of the DataView") => Some(FontError::Bounds),
            Some("This is a CFF2 font, but FDArray information is missing") => Some(
                FontError::Invalid("This is a CFF2 font, but FDArray information is missing"),
            ),
            Some("Cannot read properties of undefined (reading 'compression')") => {
                Some(FontError::MissingTable("name"))
            }
            Some("Invalid b0 31") => Some(FontError::CffOperand(31)),
            Some("Invalid b0 255") => Some(FontError::CffOperand(255)),
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
        if !phase.is_null() {
            assert_eq!(
                case["outcome"]["status"], "error",
                "constructor {}",
                case["name"]
            );
        }
    }
    assert_eq!((exact, observations), (199, 265));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
pub fn assert_previous() {
    previous::assert_corpus();
    previous::assert_previous();
}
