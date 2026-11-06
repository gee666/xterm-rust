use super::*;
use serde_json::Value;

#[test]
fn source_table_phases_before_later_constructor_checks() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../tests/fixtures/font-cff-fdselect.json")).unwrap();
    let mut count = 0;
    for case in fixture["cases"].as_array().unwrap() {
        let Some(bytes) = case["data"].as_array() else {
            continue;
        };
        let bytes: Vec<_> = bytes
            .iter()
            .map(|b| u8::try_from(b.as_u64().unwrap()).unwrap())
            .collect();
        let glyphs = if case["remove"]
            .as_array()
            .is_some_and(|r| r.iter().any(|tag| tag == "maxp"))
        {
            None
        } else {
            Some(1629)
        };
        let actual = validate_all(
            Data {
                reader: Reader::new(&bytes),
                compressed: None,
            },
            0,
            glyphs,
        )
        .err()
        .map(|error| match error {
            FontError::Bounds => "Offset is outside the bounds of the DataView".into(),
            error => error.to_string(),
        });
        assert_eq!(
            actual.as_deref(),
            case["phaseTable"]["message"].as_str(),
            "{}",
            case["name"]
        );
        count += 1;
    }
    assert_eq!(count, 273);
}
#[test]
fn absent_and_zero_glyph_counts_are_distinct_for_zero_ranges() {
    let data = Data {
        reader: Reader::new(&[3, 0, 0, 0, 0]),
        compressed: None,
    };
    let offset = address::IndexOffset::Number(0.0);
    assert_eq!(fdselect::validate(&data, 0, &offset, None, 1), Ok(()));
    assert_eq!(
        fdselect::validate(&data, 0, &offset, Some(0), 1),
        Err(FontError::CffFdSelect(
            super::super::super::font_error::FdSelectError::Sentinel(None)
        ))
    );
}
