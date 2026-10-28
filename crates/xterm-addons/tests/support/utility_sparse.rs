//! Same source-derived sparse-row assertions run natively and on WASM32.
use std::{cell::RefCell, rc::Rc};
use xterm_addons::web_links::*;
use xterm_core::{buffer::BufferLine, terminal::CoreTerminal};
struct Line(Rc<RefCell<BufferLine>>);
impl LinkLine for Line {
    fn is_wrapped(&self) -> bool {
        false
    }
    fn translate_trimmed(&self) -> Vec<u16> {
        self.0.borrow().translate_to_string(true, None, None, None)
    }
    fn len(&self) -> usize {
        self.0.borrow().len()
    }
    fn cell(&self, column: usize) -> Option<LinkCell> {
        let cell = self.0.borrow().get_cell(column)?;
        Some(LinkCell {
            chars: cell.get_chars(),
            width: u8::try_from(cell.get_width()).unwrap(),
        })
    }
}
struct Sparse {
    row: usize,
    line: Line,
}
impl LinkBuffer for Sparse {
    fn line(&self, index: usize) -> Option<&dyn LinkLine> {
        if index == self.row {
            Some(&self.line)
        } else {
            None
        }
    }
}
pub fn assert_source_cases(matcher: &dyn LinkMatcher) {
    let cases: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/utility-sparse-links.json")).unwrap();
    for case in cases.as_array().unwrap() {
        let row = usize::try_from(case["row"].as_u64().unwrap()).unwrap();
        let mut engine = CoreTerminal::new(18, 1, 0).unwrap();
        engine.write("http://example.com").unwrap();
        let line = {
            let state = engine.state();
            let buffer = state.active_buffer();
            let line = buffer.borrow().lines.get(0).unwrap();
            line
        };
        let sparse = Sparse {
            row,
            line: Line(line),
        };
        assert_eq!(
            windowed_lines(row, &sparse).1,
            case["top"].as_i64().unwrap()
        );
        let links = compute_links(row + 1, &RegexSpec::default(), &sparse, matcher).unwrap();
        assert_eq!(links.len(), 1);
        let expected = &case["range"];
        assert_eq!(
            links[0].range.start.x,
            expected["start"]["x"].as_u64().unwrap()
        );
        assert_eq!(
            links[0].range.start.y,
            expected["start"]["y"].as_u64().unwrap()
        );
        assert_eq!(links[0].range.end.x, expected["end"]["x"].as_u64().unwrap());
        assert_eq!(links[0].range.end.y, expected["end"]["y"].as_u64().unwrap());
    }
}
