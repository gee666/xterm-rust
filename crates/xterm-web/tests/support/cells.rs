use serde_json::Value;
use xterm_core::buffer::{AttributeData, BufferLine, CellData, ExtendedAttrs};
pub fn number(value: &Value) -> usize {
    value.as_u64().unwrap() as usize
}
pub fn line(cells: &[Value]) -> BufferLine {
    let mut line = BufferLine::new(cells.len(), None, false);
    for (column, raw) in cells.iter().enumerate() {
        line.set_cell(
            column,
            &CellData {
                content: number(&raw["content"]) as u32,
                attributes: AttributeData {
                    fg: number(&raw["fg"]) as u32,
                    bg: number(&raw["bg"]) as u32,
                    extended: ExtendedAttrs::new(
                        number(&raw["ext"]) as u32,
                        raw["url"].as_u64().unwrap(),
                    ),
                },
                combined_data: raw["chars"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| number(v) as u16)
                    .collect(),
            },
        );
    }
    line
}
