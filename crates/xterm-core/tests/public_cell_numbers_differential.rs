use xterm_core::{
    buffer::{
        constants::{attributes, bg_flags, content, fg_flags},
        AttributeData, BufferLine, CellData, ExtendedAttrs,
    },
    public_api::line_numbers::{get_cell_number, NumericCell, NumericCellError},
};

fn prepared_line() -> BufferLine {
    let mut line = BufferLine::new(5, None, false);
    for (index, cp, width, fg, bg) in [
        (0, 65, 1, content::IS_COMBINED_MASK | 65, 0),
        (
            1,
            content::IS_COMBINED_MASK,
            1,
            fg_flags::UNDERLINE | attributes::CM_RGB | 0x112233,
            bg_flags::HAS_EXTENDED,
        ),
        (2, 0x6587, 2, 0, 0),
        (3, 0, 0, 0, 0),
        (4, 0, 1, 0, 0),
    ] {
        let cell = CellData {
            content: cp | (width << content::WIDTH_SHIFT),
            attributes: AttributeData {
                fg,
                bg,
                extended: ExtendedAttrs::new((3 << 26) | attributes::CM_RGB | 0xabcdef, 17),
            },
            combined_data: if index == 1 {
                vec![101, 0x301]
            } else {
                Vec::new()
            },
        };
        line.set_cell(index, &cell);
    }
    line
}
fn optional(value: Option<u32>) -> String {
    value.map_or_else(|| "u".into(), |value| value.to_string())
}
fn boolean(value: bool) -> String {
    if value {
        "t".into()
    } else {
        "f".into()
    }
}
fn fallible(value: Result<String, NumericCellError>) -> String {
    value.unwrap_or_else(|_| "error:TypeError".into())
}
fn extended(cell: &NumericCell, value: String) -> String {
    fallible(cell.require_extended().map(|()| value))
}
fn snapshot(cell: &NumericCell) -> String {
    let normal = cell.normalized();
    let mut values = vec![
        optional(cell.content),
        optional(cell.fg),
        optional(cell.bg),
        cell.extended.as_ref().map_or_else(
            || "u".into(),
            |ext| format!("{},{}", ext.ext(), ext.url_id()),
        ),
        cell.chars().map_or_else(
            || "u".into(),
            |chars| {
                if chars.is_empty() {
                    "-".into()
                } else {
                    chars
                        .iter()
                        .map(|unit| format!("{unit:x}"))
                        .collect::<Vec<_>>()
                        .join(",")
                }
            },
        ),
        fallible(
            cell.code()
                .map(|code| code.map_or_else(|| "nan".into(), |code| code.to_string())),
        ),
        normal.get_width().to_string(),
        normal.get_fg_color_mode().to_string(),
        normal.get_bg_color_mode().to_string(),
        normal.get_fg_color().to_string(),
        normal.get_bg_color().to_string(),
        normal.is_bold().to_string(),
        normal.is_italic().to_string(),
        normal.is_dim().to_string(),
        extended(cell, normal.is_underline().to_string()),
        normal.is_blink().to_string(),
        normal.is_inverse().to_string(),
        normal.is_invisible().to_string(),
        normal.is_strikethrough().to_string(),
        normal.is_overline().to_string(),
        boolean(normal.is_fg_rgb()),
        boolean(normal.is_fg_palette()),
        boolean(normal.is_fg_default()),
        boolean(normal.is_bg_rgb()),
        boolean(normal.is_bg_palette()),
        boolean(normal.is_bg_default()),
        boolean(cell.is_attribute_default()),
        fallible(
            cell.require_underline_style()
                .map(|()| normal.get_underline_style().to_string()),
        ),
        extended(cell, normal.get_underline_color().to_string()),
        extended(cell, normal.get_underline_color_mode().to_string()),
        extended(cell, boolean(normal.is_underline_color_rgb())),
        extended(cell, boolean(normal.is_underline_color_palette())),
        extended(cell, boolean(normal.is_underline_color_default())),
    ];
    values.push(fallible(cell.attributes_equals(cell).map(boolean)));
    values.push(fallible(
        cell.attributes_equals(&CellData::default().into())
            .map(boolean),
    ));
    values.join("|")
}

#[test]
fn all_numeric_reads_and_cell_getters_match_untouched_source() {
    let line = prepared_line();
    let mut count = 0;
    for row in include_str!("fixtures/public-cell-numbers.tsv").lines() {
        let (bits, expected) = row.split_once('\t').unwrap();
        let index = f64::from_bits(u64::from_str_radix(bits, 16).unwrap());
        let actual = get_cell_number(&line, index)
            .as_ref()
            .map_or_else(|| "none".into(), snapshot);
        assert_eq!(actual, expected, "index bits {bits}");
        count += 1;
    }
    assert_eq!(count, 2071);
}

fn parse_number(value: &str) -> Option<f64> {
    if value == "u" {
        None
    } else {
        Some(f64::from_bits(u64::from_str_radix(value, 16).unwrap()))
    }
}

#[test]
fn numeric_ranges_and_column_maps_match_untouched_source() {
    use xterm_core::public_api::line_translation::{
        translate_to_string_number, NumericTranslationError,
    };
    let line = prepared_line();
    let mut count = 0;
    for row in include_str!("fixtures/public-line-number-translations.tsv").lines() {
        let fields: Vec<_> = row.split('\t').collect();
        let with_columns: bool = fields[3].parse().unwrap();
        let result = translate_to_string_number(
            &line,
            fields[2].parse().unwrap(),
            parse_number(fields[0]),
            parse_number(fields[1]),
            with_columns,
        );
        let actual = match result {
            Ok(translated) => {
                let text = if translated.text.is_empty() {
                    "-".into()
                } else {
                    translated
                        .text
                        .iter()
                        .map(|unit| format!("{unit:x}"))
                        .collect::<Vec<_>>()
                        .join(",")
                };
                let columns = if with_columns {
                    translated
                        .columns
                        .iter()
                        .map(|number| format!("{:016x}", number.to_bits()))
                        .collect::<Vec<_>>()
                        .join(",")
                } else {
                    "-".into()
                };
                format!("{text}|{columns}")
            }
            Err(NumericTranslationError::MissingCombinedString) => "error:TypeError".into(),
            Err(error) => panic!("unexpected translation failure {error}: {row}"),
        };
        assert_eq!(actual, fields[4], "translation {row}");
        count += 1;
    }
    assert_eq!(count, 660);
}

#[test]
fn impossible_ranges_return_a_typed_error_instead_of_hanging() {
    use xterm_core::public_api::line_translation::{
        translate_to_string_number, NumericTranslationError,
    };
    let line = prepared_line();
    assert!(matches!(
        translate_to_string_number(&line, false, Some(f64::NEG_INFINITY), Some(1.0), false),
        Err(NumericTranslationError::NonProgressingRange)
    ));
    assert!(matches!(
        translate_to_string_number(&line, false, Some(0.0), Some(f64::INFINITY), false),
        Err(NumericTranslationError::NonProgressingRange)
    ));
}
