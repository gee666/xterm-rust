//! A host warning can replace the live buffer without replacing the captured row.
use std::cell::{Cell, RefCell};
use xterm_core::{
    buffer::{AttributeData, CellData, CharData},
    serialize::{
        serialize_buffer, serialize_html, BufferAccess, CellRange, HtmlConfig, HtmlFontSize,
        HtmlOptions, Position, SerializeError,
    },
};

struct ReplacingRows {
    replaced: Cell<bool>,
    captured: RefCell<Vec<Option<CellData>>>,
    captures: RefCell<Vec<usize>>,
    warnings: RefCell<Vec<(usize, usize)>>,
    fail_capture: bool,
    fail_cell: bool,
}
fn cell(c: u16) -> Option<CellData> {
    Some(CellData::from_char_data(&CharData {
        attr: 0,
        chars: vec![c],
        width: 1,
        code: Some(u32::from(c)),
    }))
}
impl ReplacingRows {
    fn new() -> Self {
        Self {
            replaced: Cell::new(false),
            captured: RefCell::default(),
            captures: RefCell::default(),
            warnings: RefCell::default(),
            fail_capture: false,
            fail_cell: false,
        }
    }
    fn row(&self, row: usize) -> Vec<Option<CellData>> {
        if row == 0 && !self.replaced.get() {
            vec![cell(65), None, cell(67)]
        } else if row == 0 {
            vec![cell(68), cell(69), cell(70)]
        } else {
            vec![cell(71), cell(72), cell(73)]
        }
    }
}
impl BufferAccess for ReplacingRows {
    fn len(&self) -> usize {
        2
    }
    fn line_len(&self, _: usize) -> Result<Option<usize>, SerializeError> {
        Ok(Some(3))
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        Ok(self.row(row).get(col).cloned().flatten())
    }
    fn is_wrapped(&self, _: usize) -> Result<bool, SerializeError> {
        Ok(false)
    }
    fn cursor_x(&self) -> usize {
        3
    }
    fn cursor_y(&self) -> usize {
        1
    }
    fn base_y(&self) -> usize {
        0
    }
    fn capture_line(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        if self.fail_capture {
            return Err(SerializeError::BorrowedLine);
        }
        self.captures.borrow_mut().push(row);
        *self.captured.borrow_mut() = self.row(row);
        Ok(Some(3))
    }
    fn captured_cell(&self, _: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        if self.fail_cell {
            return Err(SerializeError::BorrowedLine);
        }
        Ok(self.captured.borrow().get(col).cloned().flatten())
    }
    fn missing_cell(&self, row: usize, col: usize) -> Result<(), SerializeError> {
        self.warnings.borrow_mut().push((row, col));
        self.replaced.set(true);
        Ok(())
    }
}
fn range() -> CellRange {
    CellRange {
        start: Position { x: 0, y: 0 },
        end: Position { x: 3, y: 1 },
    }
}
fn ansi(view: &ReplacingRows) -> Result<Vec<u16>, SerializeError> {
    serialize_buffer(view, 3, 2, &AttributeData::default(), range(), true)
}
fn html(view: &ReplacingRows) -> Result<Vec<u16>, SerializeError> {
    serialize_html(
        view,
        3,
        2,
        Some(range()),
        &HtmlConfig {
            foreground: None,
            background: None,
            font_family: vec![],
            font_size: HtmlFontSize::Number(15.0),
            ansi_colors: None,
        },
        &HtmlOptions {
            only_selection: true,
            ..HtmlOptions::default()
        },
    )
}
#[test]
fn ansi_keeps_captured_cells_after_warning_and_captures_the_next_row() {
    let view = ReplacingRows::new();
    assert_eq!(
        String::from_utf16(&ansi(&view).unwrap()).unwrap(),
        "AC\r\nGHI"
    );
    assert_eq!(*view.captures.borrow(), [0, 1]);
    assert_eq!(*view.warnings.borrow(), [(0, 1)]);
    assert_eq!(view.cell(0, 2).unwrap().unwrap().get_chars(), vec![70]);
}
#[test]
fn html_keeps_captured_cells_after_warning_and_captures_the_next_row() {
    let view = ReplacingRows::new();
    let output = String::from_utf16(&html(&view).unwrap()).unwrap();
    assert!(output.contains("<div><span>AC</span></div><div><span>GHI</span></div>"));
    assert_eq!(*view.captures.borrow(), [0, 1]);
    assert_eq!(*view.warnings.borrow(), [(0, 1)]);
}
#[test]
fn capture_and_cell_failures_propagate_before_host_warnings() {
    for fail_capture in [true, false] {
        for serialize in [ansi, html] {
            let mut view = ReplacingRows::new();
            view.fail_capture = fail_capture;
            view.fail_cell = !fail_capture;
            assert_eq!(serialize(&view), Err(SerializeError::BorrowedLine));
            assert!(view.warnings.borrow().is_empty());
        }
    }
}
