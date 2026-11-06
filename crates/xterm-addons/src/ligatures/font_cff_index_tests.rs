use super::*;
use serde_json::Value;

pub(super) fn expected(value: &Value) -> Option<FontError> {
    match value["message"].as_str() {
        None => None,
        Some("Offset is outside the bounds of the DataView") => Some(FontError::Bounds),
        Some("This is a CFF2 font, but FDArray information is missing") => Some(
            FontError::Invalid("This is a CFF2 font, but FDArray information is missing"),
        ),
        Some("Invalid b0 31") => Some(FontError::CffOperand(31)),
        Some("Invalid b0 255") => Some(FontError::CffOperand(255)),
        Some(message) if message.starts_with("Object [object Object] already has key ") => Some(
            FontError::CffDuplicate(message.rsplit(' ').next().unwrap().parse().unwrap()),
        ),
        value => panic!("Unmapped source phase {value:?}"),
    }
}
#[test]
fn unchanged_source_fdarray_index_phase_outcomes() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../tests/fixtures/font-cff2-index.json")).unwrap();
    let mut exact = 0;
    let mut observations = 0;
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        if !case["data"].is_array() {
            continue;
        }
        let bytes: Vec<_> = case["data"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
            .collect();
        let actual = validate_data(
            Data {
                reader: Reader::new(&bytes),
                compressed: None,
            },
            0,
        )
        .err();
        if case["phaseTable"].is_object() {
            exact += 1;
            let expected = expected(&case["phaseTable"]);
            if actual != expected {
                failures.push(format!(
                    "{} expected {expected:?}, got {actual:?}",
                    case["name"]
                ));
            }
        } else {
            // A source child failed its resource bound before this phase could
            // report. Rust must finish, but this is not an equivalence assertion.
            assert!(case["resourceFailure"].is_object());
            observations += 1;
        }
    }
    assert_eq!((exact, observations), (450, 10));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn data_view_to_index_preserves_fractional_and_nan_boundaries() {
    use address::to_index;
    for (input, expected) in [(f64::NAN, 0), (-0.0, 0), (-0.5, 0), (1.9, 1)] {
        assert_eq!(to_index(input), Ok(expected));
    }
    for value in [
        f64::INFINITY,
        f64::NEG_INFINITY,
        -1.0,
        9_007_199_254_740_992.0,
    ] {
        assert_eq!(to_index(value), Err(FontError::Bounds));
    }
}
#[test]
fn empty_reversed_and_forward_numeric_index_ranges() {
    for bytes in [
        vec![0, 0, 0, 0],
        vec![0, 0, 0, 1, 0],
        vec![0, 0, 0, 1, 1, 255, 1],
        vec![0, 0, 0, 1, 1, 255, 255],
        vec![0, 0, 0, 1, 1, 1, 2, 0],
    ] {
        assert!(index(
            &Data {
                reader: Reader::new(&bytes),
                compressed: None
            },
            0.0,
            true
        )
        .is_ok());
    }
    assert_eq!(
        index(
            &Data {
                reader: Reader::new(&[0, 0, 0, 1, 1, 1, 2]),
                compressed: None
            },
            0.0,
            true
        ),
        Err(FontError::Bounds)
    );
}
