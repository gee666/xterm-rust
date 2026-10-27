use std::rc::Rc;
use xterm_core::{
    buffer::{AttributeData, Buffer},
    options::OptionsService,
    serialize::*,
};

fn buffer() -> Buffer {
    let options = Rc::new(OptionsService::new(
        false,
        Vec::<(&str, xterm_core::options::OptionValue)>::new(),
    ));
    let mut b = Buffer::new(true, 10, 2, options).unwrap();
    b.fill_viewport_rows(None).unwrap();
    b
}
fn serialize(b: &Buffer) -> Result<Vec<u16>, SerializeError> {
    serialize_buffer(
        b,
        10,
        2,
        &AttributeData::default(),
        CellRange {
            start: Position { x: 0, y: 0 },
            end: Position { x: 10, y: 1 },
        },
        false,
    )
}
#[test]
fn native_buffer_reads_live_cells_without_detached_storage() {
    let mut b = buffer();
    assert!(serialize(&b).unwrap().is_empty());
    b.lines
        .get(0)
        .unwrap()
        .borrow_mut()
        .set_cell_from_codepoint(0, u32::from('x'), 1, &AttributeData::default());
    b.x = 1;
    assert_eq!(serialize(&b).unwrap(), vec![u16::from(b'x')]);
    b.lines
        .get(0)
        .unwrap()
        .borrow_mut()
        .set_cell_from_codepoint(0, u32::from('y'), 1, &AttributeData::default());
    assert_eq!(serialize(&b).unwrap(), vec![u16::from(b'y')]);
}
#[test]
fn mutable_line_borrow_propagates_instead_of_panicking_or_losing_cells() {
    let b = buffer();
    let row = b.lines.get(0).unwrap();
    let guard = row.borrow_mut();
    assert_eq!(b.line_len(0), Err(SerializeError::BorrowedLine));
    assert!(matches!(b.cell(0, 0), Err(SerializeError::BorrowedLine)));
    assert_eq!(b.is_wrapped(0), Err(SerializeError::BorrowedLine));
    assert_eq!(serialize(&b), Err(SerializeError::BorrowedLine));
    drop(guard);
    assert!(serialize(&b).unwrap().is_empty());
}
#[test]
fn absent_native_lines_remain_distinct_from_borrow_errors() {
    let b = buffer();
    assert_eq!(b.line_len(99).unwrap(), None);
    assert!(b.cell(99, 0).unwrap().is_none());
    assert!(!b.is_wrapped(99).unwrap());
    assert_eq!(BufferAccess::len(&b), 2);
    assert_eq!(b.base_y(), 0);
    assert_eq!(b.cursor_y(), 0);
}
