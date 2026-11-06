use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};
#[cfg(target_arch = "wasm32")]
#[path = "font_cff_container.rs"]
mod container;
#[cfg(not(target_arch = "wasm32"))]
#[path = "font_cff2_index_assertions.rs"]
mod index_assertions;
#[cfg(target_arch = "wasm32")]
pub use container::construct;
#[cfg(not(target_arch = "wasm32"))]
pub use index_assertions::construct;

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-cff2-fonts.json")).unwrap()
}
pub fn expected(value: &Value) -> Option<FontError> {
    match value["message"].as_str() {
        None => None,
        Some("Offset is outside the bounds of the DataView") => Some(FontError::Bounds),
        Some("Cannot read properties of undefined (reading 'compression')") => {
            Some(FontError::MissingTable("name"))
        }
        Some("Invalid b0 31") => Some(FontError::CffOperand(31)),
        Some("Invalid b0 255") => Some(FontError::CffOperand(255)),
        Some(message) if message.starts_with("Object [object Object] already has key ") => Some(
            FontError::CffDuplicate(message.rsplit(' ').next().unwrap().parse().unwrap()),
        ),
        message => panic!("Unmapped source constructor {message:?}"),
    }
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
        if case["outcome"]["status"] == "resourceFailure" {
            observations += 1;
            continue;
        }
        let expected = expected(&case["outcome"]);
        if actual != expected {
            failures.push(format!(
                "{} expected {expected:?}, got {actual:?}",
                case["name"]
            ));
        }
        exact += 1;
    }
    assert_eq!((exact, observations), (396, 12));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[cfg(not(target_arch = "wasm32"))]
pub fn assert_previous() {
    index_assertions::assert_corpus();
    index_assertions::assert_previous();
}
