// Copyright (c) 2017 The xterm.js authors. All rights reserved.
// Licensed under MIT. Assertions ported from SelectionModel.test.ts.
use std::num::NonZeroU32;
use xterm_web::selection::model::{CoordinateOverflow, Dimensions, Position, SelectionModel};

fn point(column: u32, row: i64) -> Option<Position> {
    Some(Position::new(column, row))
}
fn dimensions() -> Dimensions {
    Dimensions {
        columns: NonZeroU32::new(80).unwrap(),
        rows: NonZeroU32::new(2).unwrap(),
        scrollback: 0,
    }
}
fn model(start: Option<Position>, end: Option<Position>, length: u32) -> SelectionModel {
    SelectionModel {
        selection_start: start,
        selection_end: end,
        selection_start_length: length,
        ..SelectionModel::default()
    }
}
fn end(model: &SelectionModel) -> Option<Position> {
    model.final_selection_end(dimensions()).unwrap()
}

#[test]
fn clear_selection_should_clear_the_final_selection() {
    let mut model = model(point(0, 0), point(10, 2), 0);
    assert_eq!(model.final_selection_start(), point(0, 0));
    assert_eq!(end(&model), point(10, 2));
    model.clear_selection();
    assert_eq!(model.final_selection_start(), None);
    assert_eq!(end(&model), None);
}
#[test]
fn reversed_when_end_precedes_start() {
    let mut model = model(point(1, 0), point(0, 0), 0);
    assert!(model.are_selection_values_reversed());
    model.selection_start = point(10, 2);
    assert!(model.are_selection_values_reversed());
}
#[test]
fn not_reversed_when_end_follows_start() {
    let mut model = model(point(0, 0), point(1, 0), 0);
    assert!(!model.are_selection_values_reversed());
    model.selection_end = point(10, 2);
    assert!(!model.are_selection_values_reversed());
}
#[test]
fn trim_a_portion_of_selection() {
    let mut model = model(point(0, 0), point(10, 2), 0);
    model.handle_trim(1).unwrap();
    assert_eq!(model.final_selection_start(), point(0, 0));
    assert_eq!(end(&model), point(10, 1));
    model.handle_trim(1).unwrap();
    assert_eq!(model.final_selection_start(), point(0, 0));
    assert_eq!(end(&model), point(10, 0));
}
#[test]
fn clear_when_trimmed_in_entirety() {
    let mut model = model(point(0, 0), point(10, 0), 0);
    model.handle_trim(1).unwrap();
    assert_eq!(model.final_selection_start(), None);
    assert_eq!(end(&model), None);
}
#[test]
fn reset_start_to_origin_when_start_row_trimmed() {
    let mut model = model(point(50, 0), point(10, 2), 0);
    assert!(model.handle_trim(1).unwrap());
    assert_eq!(model.final_selection_start(), point(0, 0));
    assert_eq!(end(&model), point(10, 1));
}
#[test]
fn select_all_starts_at_origin() {
    let model = SelectionModel {
        is_select_all_active: true,
        ..SelectionModel::default()
    };
    assert_eq!(model.final_selection_start(), point(0, 0));
}
#[test]
fn final_start_without_end() {
    assert_eq!(
        model(point(2, 2), None, 0).final_selection_start(),
        point(2, 2)
    );
}
#[test]
fn final_start_reverses_values() {
    let mut model = model(point(2, 2), point(3, 2), 0);
    assert_eq!(model.final_selection_start(), point(2, 2));
    model.selection_end = point(1, 2);
    assert_eq!(model.final_selection_start(), point(1, 2));
}
#[test]
fn select_all_ends_at_buffer_end() {
    let model = SelectionModel {
        is_select_all_active: true,
        ..SelectionModel::default()
    };
    assert_eq!(end(&model), point(80, 1));
}
#[test]
fn no_end_without_start() {
    let mut model = SelectionModel::default();
    assert_eq!(end(&model), None);
    model.selection_end = point(1, 2);
    assert_eq!(end(&model), None);
}
#[test]
fn end_uses_start_plus_length_without_end() {
    assert_eq!(end(&model(point(2, 2), None, 2)), point(4, 2));
}
#[test]
fn end_uses_start_plus_length_when_reversed() {
    assert_eq!(end(&model(point(2, 2), point(2, 1), 2)), point(4, 2));
}
#[test]
fn end_inside_minimum_uses_minimum() {
    assert_eq!(end(&model(point(2, 2), point(3, 2), 2)), point(4, 2));
}
#[test]
fn overflow_to_following_row() {
    assert_eq!(end(&model(point(78, 2), None, 4)), point(2, 3));
}
#[test]
fn overflow_with_end_in_between() {
    assert_eq!(end(&model(point(78, 2), point(79, 2), 4)), point(2, 3));
}
#[test]
fn explicit_end_after_minimum_wins() {
    assert_eq!(end(&model(point(2, 2), point(5, 2), 2)), point(5, 2));
}
#[test]
fn no_trailing_eol_at_end_of_line() {
    let mut model = model(point(0, 0), None, 80);
    assert_eq!(end(&model), point(80, 0));
    model.selection_start_length = 160;
    assert_eq!(end(&model), point(80, 1));
}

// Additional boundaries absent from the upstream unit suite.
#[test]
fn exact_wrap_with_explicit_same_row_end_preserves_source_branch() {
    assert_eq!(end(&model(point(78, 2), point(79, 2), 82)), point(0, 4));
    assert_eq!(end(&model(point(78, 2), None, 82)), point(80, 3));
}
#[test]
fn trim_returns_false_for_coordinate_only_move() {
    let mut model = model(point(2, 2), point(5, 3), 2);
    assert!(!model.handle_trim(1).unwrap());
    assert_eq!(model.selection_start, point(2, 1));
    assert_eq!(model.selection_end, point(5, 2));
    model.is_select_all_active = true;
    model.clear_selection();
    assert_eq!(model, SelectionModel::default());
}
#[test]
fn dimensions_are_live_for_select_all() {
    let model = SelectionModel {
        is_select_all_active: true,
        ..SelectionModel::default()
    };
    let mut dimensions = dimensions();
    dimensions.columns = NonZeroU32::new(132).unwrap();
    dimensions.rows = NonZeroU32::new(24).unwrap();
    dimensions.scrollback = 1000;
    assert_eq!(
        model.final_selection_end(dimensions).unwrap(),
        point(132, 1023)
    );
}
#[test]
fn numeric_overflow_is_recoverable_and_trim_atomic() {
    let mut model = model(point(0, i64::MIN), point(1, 0), 0);
    let original = model.clone();
    assert_eq!(model.handle_trim(1), Err(CoordinateOverflow));
    assert_eq!(model, original);
    model.selection_start = point(79, i64::MAX);
    model.selection_end = None;
    model.selection_start_length = 2;
    assert_eq!(
        model.final_selection_end(dimensions()),
        Err(CoordinateOverflow)
    );
}
