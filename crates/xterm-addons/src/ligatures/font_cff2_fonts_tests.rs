use super::*;
use serde_json::Value;

#[test]
fn unchanged_source_eager_font_private_and_local_subrs_phase() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../tests/fixtures/font-cff2-fonts.json")).unwrap();
    let mut count = 0;
    let mut observations = 0;
    let mut failures = Vec::new();
    for case in fixture["cases"].as_array().unwrap() {
        let Some(values) = case["data"].as_array() else {
            continue;
        };
        let bytes: Vec<_> = values
            .iter()
            .map(|v| u8::try_from(v.as_u64().unwrap()).unwrap())
            .collect();
        let actual = validate_fonts(
            Data {
                reader: Reader::new(&bytes),
                compressed: None,
            },
            0,
        )
        .err();
        if !case["phaseTable"].is_object() {
            assert_eq!(case["resourceFailure"]["kind"], "heapLimit");
            observations += 1;
            continue;
        }
        let expected = tests::expected(&case["phaseTable"]);
        if actual != expected {
            failures.push(format!(
                "{} expected {expected:?}, got {actual:?}",
                case["name"]
            ));
        }
        count += 1;
    }
    assert_eq!((count, observations), (392, 12));
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
#[test]
fn nonterminating_private_size_is_an_explicit_safety_divergence() {
    let mut font = vec![30, 0x1b, 0x99, 0x99, 0xff, 30, 0xff, 18];
    let data = Data {
        reader: Reader::new(&[2, 0, 0, 0]),
        compressed: None,
    };
    assert_eq!(
        fonts::validate(&data, 0.0, &font),
        Err(FontError::Invalid(
            "CFF2 private dictionary does not terminate"
        ))
    );
    let invalid = Data {
        reader: Reader::new(&[31]),
        compressed: None,
    };
    assert_eq!(
        fonts::validate(&invalid, 0.0, &font),
        Err(FontError::CffOperand(31))
    );
    // A negative size never enters the loop, even with an invalid offset.
    font = vec![138, 30, 0xff, 18];
    assert_eq!(fonts::validate(&invalid, 0.0, &font), Ok(()));
}
#[test]
fn multi_operand_subrs_terminates_without_claiming_source_resource_equivalence() {
    let data = Data {
        reader: Reader::new(&[2, 0, 0, 0]),
        compressed: None,
    };
    assert_eq!(
        address::IndexOffset::operands(&[0.0, 1.0])
            .unwrap()
            .subrs(&data, 256.0, 0.0),
        Ok(())
    );
}
