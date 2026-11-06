use serde_json::Value;
use xterm_addons::ligatures::font::{Font, FontError, FontOptions};
#[cfg(target_arch = "wasm32")]
#[path = "font_cff_container.rs"]
mod container;
#[cfg(target_arch = "wasm32")]
pub use container::construct;
#[cfg(not(target_arch = "wasm32"))]
#[path = "font_cff2_fonts_assertions.rs"]
mod previous;
#[cfg(not(target_arch = "wasm32"))]
pub use previous::construct;
#[cfg(not(target_arch = "wasm32"))]
pub fn assert_previous() {
    previous::assert_corpus();
    previous::assert_previous();
}

pub fn fixture() -> Value {
    serde_json::from_str(include_str!("../fixtures/font-cff-fdselect.json")).unwrap()
}
pub fn canonical(error: FontError) -> String {
    match error {
        FontError::Bounds => "Offset is outside the bounds of the DataView".into(),
        FontError::MissingTable("name" | "hmtx") => {
            "Cannot read properties of undefined (reading 'compression')".into()
        }
        error => error.to_string(),
    }
}
pub fn assert_corpus() {
    let fixture = fixture();
    let mut failures = Vec::new();
    let cases = fixture["cases"].as_array().unwrap();
    for case in cases {
        let bytes = construct(case, &fixture);
        assert_eq!(
            u64::from(bytes.iter().fold(2166136261_u32, |h, b| {
                (h ^ u32::from(*b)).wrapping_mul(16777619)
            })),
            case["inputFnv32"].as_u64().unwrap(),
            "input {}",
            case["name"]
        );
        let actual = Font::load_buffer(&bytes, FontOptions::default())
            .err()
            .map(canonical);
        let expected = case["outcome"]["message"].as_str();
        if actual.as_deref() != expected {
            failures.push(format!(
                "{} expected {expected:?}, got {actual:?}",
                case["name"]
            ));
        }
    }
    assert_eq!(cases.len(), 277);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
