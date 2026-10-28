use xterm_core::{
    buffer::{
        constants::{bg_flags, content},
        BufferLine, CellData,
    },
    public_api::line_numbers::{get_cell_number, NumericCellError},
};

#[test]
fn numeric_cell_preserves_nan_missing_words_and_strict_defaults() {
    let line = BufferLine::new(2, None, false);
    let cell = get_cell_number(&line, f64::NAN).unwrap();
    assert!(cell.content.is_none() && cell.fg.is_none() && cell.bg.is_none());
    assert_eq!(cell.chars(), Some(Vec::new()));
    assert_eq!(cell.code(), Ok(Some(0)));
    assert_eq!(cell.normalized().get_width(), 0);
    assert!(!cell.is_attribute_default());
}

#[test]
fn fractional_cell_address_can_read_foreground_as_content() {
    let mut line = BufferLine::new(2, None, false);
    let mut value = CellData::default();
    value.attributes.fg = content::IS_COMBINED_MASK | 65;
    line.set_cell(0, &value);
    let cell = get_cell_number(&line, 1.0 / 3.0).unwrap();
    assert_eq!(cell.content, Some(content::IS_COMBINED_MASK | 65));
    assert!(cell.chars().is_none());
    assert_eq!(cell.code(), Err(NumericCellError::MissingCombinedString));
}

#[test]
fn bounds_use_original_number_and_do_not_truncate() {
    let line = BufferLine::new(2, None, false);
    for index in [
        -1.0,
        -f64::MIN_POSITIVE,
        f64::INFINITY,
        f64::NEG_INFINITY,
        2.0,
        f64::MAX,
    ] {
        assert!(get_cell_number(&line, index).is_none());
    }
    for index in [0.0, -0.0, 0.5, 1.5] {
        assert!(get_cell_number(&line, index).is_some());
    }
    assert!(get_cell_number(&line, 0.5).unwrap().content.is_none());
}

#[test]
fn misaligned_extended_flag_does_not_invent_an_extended_object() {
    let mut line = BufferLine::new(2, None, false);
    let mut value = CellData::default();
    value.attributes.fg = bg_flags::HAS_EXTENDED;
    line.set_cell(1, &value);
    let cell = get_cell_number(&line, 2.0 / 3.0).unwrap();
    assert_eq!(cell.bg, Some(bg_flags::HAS_EXTENDED));
    assert!(cell.extended.is_none());
    assert_eq!(
        cell.require_extended(),
        Err(NumericCellError::MissingExtendedAttributes)
    );
}

#[test]
fn scalar_metadata_does_not_copy_combined_text_or_extended_attributes() {
    let mut source = CellData::default();
    source.content = content::IS_COMBINED_MASK | (1 << content::WIDTH_SHIFT);
    source.combined_data = vec![0x301; 131_072];
    let cell: xterm_core::public_api::line_numbers::NumericCell = source.into();
    let metadata = cell.metadata();
    assert!(metadata.combined_data.is_empty());
    assert_eq!(metadata.get_width(), 1);
    assert!(metadata
        .attributes
        .extended
        .same_identity(cell.extended.as_ref().unwrap()));
    assert_eq!(cell.normalized().combined_data.len(), 131_072);
}
