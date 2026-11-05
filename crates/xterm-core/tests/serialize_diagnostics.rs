//! Host diagnostics belong to traversal, not lookahead reads for ANSI wrapping.
use std::cell::RefCell;
use xterm_core::{
    buffer::{AttributeData, CellData},
    serialize::{
        serialize_buffer, serialize_html, BufferAccess, CellRange, HtmlConfig, HtmlFontSize,
        HtmlOptions, Position, SerializeError,
    },
};
struct MissingCells {
    warnings: RefCell<Vec<(usize, usize)>>,
    fail: bool,
}
impl BufferAccess for MissingCells {
    fn len(&self) -> usize {
        1
    }
    fn line_len(&self, _: usize) -> Result<Option<usize>, SerializeError> {
        Ok(Some(2))
    }
    fn cell(&self, _: usize, _: usize) -> Result<Option<CellData>, SerializeError> {
        Ok(None)
    }
    fn is_wrapped(&self, _: usize) -> Result<bool, SerializeError> {
        Ok(false)
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
    fn missing_cell(&self, row: usize, col: usize) -> Result<(), SerializeError> {
        self.warnings.borrow_mut().push((row, col));
        if self.fail {
            Err(SerializeError::BorrowedLine)
        } else {
            Ok(())
        }
    }
}
fn range() -> CellRange {
    CellRange {
        start: Position { x: 0, y: 0 },
        end: Position { x: 2, y: 0 },
    }
}
#[test]
fn ansi_missing_cells_warn_in_traversal_order_and_propagate_host_failure() {
    let view = MissingCells {
        warnings: RefCell::new(Vec::new()),
        fail: false,
    };
    assert_eq!(
        serialize_buffer(&view, 2, 1, &AttributeData::default(), range(), false).unwrap(),
        Vec::<u16>::new()
    );
    assert_eq!(*view.warnings.borrow(), [(0, 0), (0, 1)]);
    let view = MissingCells {
        warnings: RefCell::new(Vec::new()),
        fail: true,
    };
    assert_eq!(
        serialize_buffer(&view, 2, 1, &AttributeData::default(), range(), false),
        Err(SerializeError::BorrowedLine)
    );
    assert_eq!(*view.warnings.borrow(), [(0, 0)]);
}
#[test]
fn html_missing_cells_warn_before_the_next_column_and_abort_on_error() {
    let config = HtmlConfig {
        foreground: None,
        background: None,
        font_family: "monospace".encode_utf16().collect(),
        font_size: HtmlFontSize::Number(15.0),
        ansi_colors: None,
    };
    let options = HtmlOptions {
        only_selection: true,
        ..HtmlOptions::default()
    };
    let view = MissingCells {
        warnings: RefCell::new(Vec::new()),
        fail: false,
    };
    let output = serialize_html(&view, 2, 1, Some(range()), &config, &options).unwrap();
    assert!(String::from_utf16(&output)
        .unwrap()
        .contains("<div><span></span></div>"));
    assert_eq!(*view.warnings.borrow(), [(0, 0), (0, 1)]);
    let view = MissingCells {
        warnings: RefCell::new(Vec::new()),
        fail: true,
    };
    assert_eq!(
        serialize_html(&view, 2, 1, Some(range()), &config, &options),
        Err(SerializeError::BorrowedLine)
    );
    assert_eq!(*view.warnings.borrow(), [(0, 0)]);
}
