//! ANSI final state must be read after a missing-cell host callback.
use std::cell::RefCell;
use xterm_core::{
    buffer::{constants::attributes::CM_P16, AttributeData, CellData, CharData},
    serialize::{serialize_buffer, BufferAccess, CellRange, Position, SerializeError},
};

struct ReentrantView {
    log: RefCell<Vec<&'static str>>,
    fail: Option<&'static str>,
    two_rows: bool,
}
impl ReentrantView {
    fn read(&self, phase: &'static str) -> Result<(), SerializeError> {
        self.log.borrow_mut().push(phase);
        if self.fail == Some(phase) {
            Err(SerializeError::BorrowedLine)
        } else {
            Ok(())
        }
    }
    fn serialize(&self, exclude_cursor: bool) -> Result<Vec<u16>, SerializeError> {
        serialize_buffer(
            self,
            2,
            1,
            &AttributeData {
                fg: CM_P16 | 1,
                ..AttributeData::default()
            },
            CellRange {
                start: Position { x: 0, y: 0 },
                end: Position {
                    x: 2,
                    y: usize::from(self.two_rows),
                },
            },
            exclude_cursor,
        )
    }
}
impl BufferAccess for ReentrantView {
    fn len(&self) -> usize {
        5
    }
    fn cursor_x(&self) -> usize {
        0
    }
    fn cursor_y(&self) -> usize {
        0
    }
    fn base_y(&self) -> usize {
        0
    }
    fn line_len(&self, _: usize) -> Result<Option<usize>, SerializeError> {
        Ok(Some(2))
    }
    fn is_wrapped(&self, _: usize) -> Result<bool, SerializeError> {
        Ok(false)
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        if col == 1 && row == usize::from(self.two_rows) {
            return Ok(None);
        }
        let chars = if row == 0 && col == 0 {
            vec![65]
        } else {
            Vec::new()
        };
        Ok(Some(CellData::from_char_data(&CharData {
            attr: 0,
            chars,
            width: 1,
            code: None,
        })))
    }
    fn missing_cell(&self, _: usize, _: usize) -> Result<(), SerializeError> {
        self.read("warn")
    }
    fn current_length(&self) -> Result<usize, SerializeError> {
        self.read("length")?;
        Ok(if self.two_rows { 2 } else { 1 })
    }
    fn current_rows(&self, _: usize) -> Result<usize, SerializeError> {
        self.read("rows")?;
        Ok(if self.two_rows { 2 } else { 1 })
    }
    fn current_cursor(&self) -> Result<Position, SerializeError> {
        self.read("cursor")?;
        Ok(Position { x: 4, y: 2 })
    }
    fn current_attrs(&self, _: &AttributeData) -> Result<AttributeData, SerializeError> {
        self.read("attrs")?;
        Ok(AttributeData {
            fg: CM_P16 | 4,
            ..AttributeData::default()
        })
    }
}
fn view(fail: Option<&'static str>, two_rows: bool) -> ReentrantView {
    ReentrantView {
        log: RefCell::new(Vec::new()),
        fail,
        two_rows,
    }
}
#[test]
fn final_metrics_cursor_and_attributes_are_read_after_warning_in_source_order() {
    let view = view(None, false);
    let output = view.serialize(false).unwrap();
    assert_eq!(
        String::from_utf16(&output).unwrap(),
        "A\x1b[2B\x1b[3C\x1b[34m"
    );
    assert_eq!(
        *view.log.borrow(),
        ["warn", "length", "rows", "cursor", "attrs"]
    );
}
#[test]
fn range_exclusion_skips_cursor_but_still_restores_live_attributes() {
    let view = view(None, false);
    assert_eq!(
        String::from_utf16(&view.serialize(true).unwrap()).unwrap(),
        "A\x1b[34m"
    );
    assert_eq!(*view.log.borrow(), ["warn", "length", "rows", "attrs"]);
}
#[test]
fn live_rows_and_length_control_trailing_blank_row_fixup() {
    let view = view(None, true);
    assert_eq!(
        String::from_utf16(&view.serialize(true).unwrap()).unwrap(),
        "A\x1b[34m"
    );
    assert_eq!(
        *view.log.borrow(),
        ["rows", "warn", "length", "rows", "attrs"]
    );
}
#[test]
fn each_late_state_failure_aborts_without_later_reads() {
    for (phase, expected) in [
        ("length", vec!["warn", "length"]),
        ("rows", vec!["warn", "length", "rows"]),
        ("cursor", vec!["warn", "length", "rows", "cursor"]),
        ("attrs", vec!["warn", "length", "rows", "cursor", "attrs"]),
    ] {
        let view = view(Some(phase), false);
        assert_eq!(view.serialize(false), Err(SerializeError::BorrowedLine));
        assert_eq!(*view.log.borrow(), expected);
    }
}
#[test]
fn row_end_reads_live_rows_and_propagates_errors_before_lookahead() {
    let view = view(Some("rows"), true);
    assert_eq!(view.serialize(true), Err(SerializeError::BorrowedLine));
    assert_eq!(*view.log.borrow(), ["rows"]);
}
