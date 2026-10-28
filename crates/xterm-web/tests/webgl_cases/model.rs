// Copyright (c) 2018 The xterm.js authors. All rights reserved.
// Licensed under MIT. RenderModel.ts storage/identity branch assertions.
use xterm_web::webgl::model::{
    CursorRenderModel, CursorShape, ModelError, RenderModel, BACKGROUND_OFFSET,
    COMBINED_CHAR_BIT_MASK, EXTENDED_OFFSET, FOREGROUND_OFFSET, INDICES_PER_CELL,
};

#[test]
fn constructor_and_packed_offsets_match_source() {
    let model = RenderModel::new();
    assert!(model.cells.is_empty());
    assert!(model.line_lengths.is_empty());
    assert!(!model.selection.has_selection);
    assert!(model.cursor.is_none());
    assert_eq!(
        (
            INDICES_PER_CELL,
            BACKGROUND_OFFSET,
            FOREGROUND_OFFSET,
            EXTENDED_OFFSET
        ),
        (4, 1, 2, 3)
    );
    assert_eq!(COMBINED_CHAR_BIT_MASK, 0x8000_0000);
}

#[test]
fn resize_allocates_zeroed_arrays_only_when_total_packed_length_changes() {
    let mut model = RenderModel::new();
    model.resize(2, 3).unwrap();
    assert_eq!(model.cells, vec![0; 24]);
    assert_eq!(model.line_lengths, [0, 0, 0]);
    model.cells[BACKGROUND_OFFSET] = 7;
    model.line_lengths[2] = 2;
    let address = model.cells.as_ptr();
    model.resize(3, 2).unwrap();
    assert_eq!(model.cells.as_ptr(), address);
    assert_eq!(model.cells[BACKGROUND_OFFSET], 7);
    // The upstream equality condition preserves the old row array too.
    assert_eq!(model.line_lengths, [0, 0, 2]);
    model.resize(3, 3).unwrap();
    assert_eq!(model.cells, vec![0; 36]);
    assert_eq!(model.line_lengths, [0, 0, 0]);
}

#[test]
fn clear_and_resize_preserve_selection_and_cursor() {
    let mut model = RenderModel::new();
    let cursor = CursorRenderModel {
        x: 1,
        y: 2,
        width: 2,
        style: CursorShape::Underline,
        cursor_width: 1.5,
        device_pixel_ratio: 1.25,
    };
    model.selection.has_selection = true;
    model.cursor = Some(cursor);
    model.resize(2, 3).unwrap();
    model.cells.fill(u32::MAX);
    model.line_lengths.fill(2);
    model.clear();
    assert!(model.cells.iter().all(|value| *value == 0));
    assert!(model.line_lengths.iter().all(|value| *value == 0));
    assert!(model.selection.has_selection);
    assert_eq!(model.cursor, Some(cursor));
    model.resize(3, 3).unwrap();
    assert!(model.selection.has_selection);
    assert_eq!(model.cursor, Some(cursor));
}

#[test]
fn zero_product_preserves_an_existing_zero_length_model() {
    let mut model = RenderModel::new();
    model.resize(0, 7).unwrap();
    assert!(model.line_lengths.is_empty());
    model.resize(1, 2).unwrap();
    model.resize(0, 7).unwrap();
    assert!(model.cells.is_empty());
    assert_eq!(model.line_lengths, vec![0; 7]);
    model.resize(0, 3).unwrap();
    assert_eq!(model.line_lengths.len(), 7);
}

#[test]
fn dimension_overflow_returns_error_without_replacing_previous_storage() {
    let mut model = RenderModel::new();
    model.resize(1, 1).unwrap();
    model.cells[0] = 99;
    assert!(matches!(
        model.resize(usize::MAX, 2),
        Err(ModelError::DimensionsOverflow)
    ));
    assert_eq!(model.cells, [99, 0, 0, 0]);
    assert_eq!(model.line_lengths, [0]);
}
