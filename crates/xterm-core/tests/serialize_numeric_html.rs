use xterm_core::{
    buffer::{CellData, LineRef},
    headless::Terminal,
    public_api::line_numbers::{get_cell_number, NumericCell},
    serialize::*,
};
struct NumericRow {
    line: LineRef,
    columns: Vec<f64>,
}
impl BufferAccess for NumericRow {
    fn len(&self) -> usize {
        1
    }
    fn cursor_x(&self) -> usize {
        3
    }
    fn cursor_y(&self) -> usize {
        0
    }
    fn base_y(&self) -> usize {
        0
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        Ok((row == 0).then_some(self.columns.len()))
    }
    fn is_wrapped(&self, _: usize) -> Result<bool, SerializeError> {
        Ok(false)
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        self.numeric_cell(row, col)
            .map(|v| v.map(|v| v.normalized()))
    }
    fn numeric_cell(&self, row: usize, col: usize) -> Result<Option<NumericCell>, SerializeError> {
        if row != 0 {
            return Ok(None);
        }
        let Some(index) = self.columns.get(col) else {
            return Ok(None);
        };
        Ok(get_cell_number(&self.line.borrow(), *index))
    }
}
fn render(input: &str, start: f64) -> Result<String, SerializeError> {
    let t = Terminal::new(3, 1, 0).unwrap();
    t.write(input).unwrap();
    let source = t.buffer().unwrap().active().unwrap();
    let line = source.get_line(0).unwrap().unwrap();
    let mut columns = Vec::new();
    let mut col = start;
    while col < 3.0 {
        columns.push(col);
        col += 1.0;
    }
    let count = columns.len();
    let view = NumericRow { line, columns };
    let config = HtmlConfig {
        foreground: None,
        background: None,
        font_family: "monospace".encode_utf16().collect(),
        font_size: 15.0.into(),
        ansi_colors: None,
    };
    let out = serialize_html_range(
        &view,
        Some(CellRange {
            start: Position { x: 0, y: 0 },
            end: Position { x: count, y: 0 },
        }),
        &config,
        false,
    )?;
    Ok(String::from_utf16(&out).unwrap())
}
fn html(row: &str) -> String {
    format!("<html><body><!--StartFragment--><pre><div style='color: #000000; background-color: #ffffff; font-family: monospace; font-size: 15px;'>{row}</div></pre><!--EndFragment--></body></html>")
}
#[test]
fn fractional_columns_preserve_packed_words_and_undefined_combined_text() {
    assert_eq!(
        render("\x1b[31mA\x1b[0mBC", 1.0 / 3.0).unwrap(),
        html("<div><span>\u{1}</span></div>")
    );
    assert_eq!(
        render("\x1b[38;2;255;18;52mA\x1b[0mBC", 1.0 / 3.0).unwrap(),
        html("<div><span>undefined</span></div>")
    );
    assert_eq!(
        render("A\x1b[4mB\x1b[0mC", 1.0 / 3.0).unwrap(),
        html("<div><span></span><span style='text-decoration: underline;'> </span></div>")
    );
}
#[test]
fn missing_extended_attributes_preserve_source_type_error() {
    let error = render("A\x1b[4mB\x1b[0mC", 2.0 / 3.0).unwrap_err();
    assert_eq!(error, SerializeError::MissingExtendedAttributes);
    assert_eq!(
        error.to_string(),
        "Cannot read properties of undefined (reading 'underlineStyle')"
    );
}
#[test]
fn nan_and_negative_columns_match_source_output() {
    assert_eq!(
        render("\x1b[31mA\x1b[0mBC", f64::NAN).unwrap(),
        html("<div><span></span></div>")
    );
    assert_eq!(
        render("\x1b[31mA\x1b[0mBC", -1.0).unwrap(),
        html("<div><span></span><span style='color: #cc0000;'>A</span><span>BC</span></div>")
    );
}
