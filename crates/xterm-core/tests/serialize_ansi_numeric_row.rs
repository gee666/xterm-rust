// Copyright (c) 2019 The xterm.js authors. All rights reserved. MIT license.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{constants::attributes::CM_P16, AttributeData, Buffer, CellData},
    options::{OptionValue, OptionsService},
    public_api::{BufferApiView, BufferType},
    serialize::{
        serialize_buffer, serialize_numeric_row, BufferAccess, CellRange, Position,
        PublicBufferView, SerializeError,
    },
};

fn ring() -> Buffer {
    let options = Rc::new(OptionsService::new(
        false,
        [("scrollback", OptionValue::Number(0.0))],
    ));
    let mut buffer = Buffer::new(true, 3, 2, options).unwrap();
    buffer.fill_viewport_rows(None).unwrap();
    buffer.lines.recycle().unwrap();
    for (row, text) in ["def", "ghi"].iter().enumerate() {
        let line = buffer.lines.get(row).unwrap();
        for (col, ch) in text.chars().enumerate() {
            line.borrow_mut().set_cell_from_codepoint(
                col,
                u32::from(ch),
                1,
                &AttributeData::default(),
            );
        }
    }
    buffer
}
fn numeric(buffer: &dyn BufferAccess, row: f64) -> String {
    String::from_utf16(
        &serialize_numeric_row(buffer, 3, 2, &AttributeData::default(), row).unwrap(),
    )
    .unwrap()
}
#[test]
fn raw_numeric_rows_keep_rotated_ring_aliases() {
    let buffer = ring();
    let expected = [
        (-1.0, "ghi"),
        (f64::from_bits(1), "def"),
        (4_294_967_296.0, "def"),
        (9_007_199_254_740_991.0, "ghi"),
        (-0.5, ""),
        (0.4, ""),
        (3.429227000030392, "undefined"),
        (1.300047319391903, "undefined"),
    ];
    for (row, text) in expected {
        assert_eq!(numeric(&buffer, row), text);
    }
    let view = PublicBufferView::new(Rc::new(BufferApiView::new(
        Rc::new(RefCell::new(buffer)),
        BufferType::Normal,
    )))
    .unwrap();
    for (row, text) in expected {
        assert_eq!(numeric(&view, row), text);
    }
}
#[test]
fn integral_and_negative_zero_match_native_single_row() {
    let buffer = ring();
    for row in 0..2 {
        let expected = serialize_buffer(
            &buffer,
            3,
            2,
            &AttributeData::default(),
            CellRange {
                start: Position { x: 0, y: row },
                end: Position { x: 3, y: row },
            },
            true,
        )
        .unwrap();
        assert_eq!(
            serialize_numeric_row(&buffer, 3, 2, &AttributeData::default(), row as f64).unwrap(),
            expected
        );
    }
    assert_eq!(numeric(&buffer, -0.0), numeric(&buffer, 0.0));
}
#[test]
fn nonfinite_and_out_of_safe_domain_rows_are_explicit_errors() {
    let buffer = ring();
    for row in [
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        9_007_199_254_740_992.0,
        -9_007_199_254_740_992.0,
    ] {
        assert_eq!(
            serialize_numeric_row(&buffer, 3, 2, &AttributeData::default(), row),
            Err(SerializeError::InvalidRange)
        );
    }
}
struct WarningView {
    buffer: Buffer,
    warnings: RefCell<Vec<(f64, usize)>>,
    attrs: RefCell<AttributeData>,
    failure: Option<SerializeError>,
}
impl BufferAccess for WarningView {
    fn len(&self) -> usize {
        self.buffer.len()
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        self.buffer.line_len(row)
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        self.buffer.cell(row, col)
    }
    fn is_wrapped(&self, row: usize) -> Result<bool, SerializeError> {
        self.buffer.is_wrapped(row)
    }
    fn cursor_x(&self) -> usize {
        99
    }
    fn cursor_y(&self) -> usize {
        99
    }
    fn base_y(&self) -> usize {
        99
    }
    fn capture_numeric_line(&self, row: f64) -> Result<Option<usize>, SerializeError> {
        self.buffer.capture_numeric_line(row)
    }
    fn captured_numeric_cell(
        &self,
        row: f64,
        col: usize,
    ) -> Result<Option<CellData>, SerializeError> {
        self.buffer.captured_numeric_cell(row, col)
    }
    fn missing_numeric_cell(&self, row: f64, col: usize) -> Result<(), SerializeError> {
        self.warnings.borrow_mut().push((row, col));
        self.attrs.borrow_mut().fg = CM_P16 | 1;
        self.failure.clone().map_or(Ok(()), Err)
    }
    fn current_attrs(&self, _: &AttributeData) -> Result<AttributeData, SerializeError> {
        Ok(self.attrs.borrow().clone())
    }
}
#[test]
fn missing_cells_keep_raw_warning_key_and_restore_late_attributes() {
    let buffer = WarningView {
        buffer: ring(),
        warnings: RefCell::default(),
        attrs: RefCell::default(),
        failure: None,
    };
    let output = serialize_numeric_row(&buffer, 5, 2, &AttributeData::default(), -1.0).unwrap();
    assert_eq!(String::from_utf16(&output).unwrap(), "ghi\x1b[31m");
    assert_eq!(*buffer.warnings.borrow(), [(-1.0, 3), (-1.0, 4)]);
}
#[test]
fn warning_failure_stops_traversal_without_late_style_success() {
    let buffer = WarningView {
        buffer: ring(),
        warnings: RefCell::default(),
        attrs: RefCell::default(),
        failure: Some(SerializeError::BorrowedLine),
    };
    assert_eq!(
        serialize_numeric_row(&buffer, 5, 2, &AttributeData::default(), -1.0),
        Err(SerializeError::BorrowedLine)
    );
    assert_eq!(*buffer.warnings.borrow(), [(-1.0, 3)]);
}
