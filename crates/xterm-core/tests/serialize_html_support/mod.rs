use xterm_core::{
    buffer::{CellData, CharData},
    serialize::{
        serialize_html, BufferAccess, CellRange, HtmlConfig, HtmlOptions, Position, SerializeError,
    },
};

pub struct Buffer {
    pub lines: Vec<Vec<CellData>>,
}
impl BufferAccess for Buffer {
    fn len(&self) -> usize {
        self.lines.len()
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        Ok(self.lines.get(row).map(Vec::len))
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        Ok(self.lines.get(row).and_then(|line| line.get(col)).cloned())
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
}
pub fn cell(text: &str) -> CellData {
    CellData::from_char_data(&CharData {
        attr: 0,
        chars: text.encode_utf16().collect(),
        width: 1,
        code: None,
    })
}
pub fn buffer(lines: &[&str], cols: usize) -> Buffer {
    Buffer {
        lines: lines
            .iter()
            .map(|line| {
                let mut cells: Vec<_> = line.chars().map(|c| cell(&c.to_string())).collect();
                cells.resize_with(cols, || cell(""));
                cells
            })
            .collect(),
    }
}
pub fn config() -> HtmlConfig {
    HtmlConfig {
        foreground: None,
        background: None,
        font_family: "monospace".encode_utf16().collect(),
        font_size: 15.0,
        ansi_colors: None,
    }
}
pub fn html(
    buffer: &dyn BufferAccess,
    cols: usize,
    options: &HtmlOptions,
    selection: Option<CellRange>,
    config: &HtmlConfig,
) -> String {
    String::from_utf16(&serialize_html(buffer, cols, 2, selection, config, options).unwrap())
        .unwrap()
}
pub fn range(x: usize, y: usize, end_x: usize, end_y: usize) -> CellRange {
    CellRange {
        start: Position { x, y },
        end: Position { x: end_x, y: end_y },
    }
}
