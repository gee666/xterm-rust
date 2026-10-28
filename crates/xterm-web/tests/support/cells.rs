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

#[test]
fn fixture_cells_preserve_wide_url_ids() {
    for url_id in [u64::from(u32::MAX) + 1, (1_u64 << 53) - 1] {
        let raw = serde_json::json!({
            "content": 0, "fg": 0,
            "bg": xterm_core::buffer::constants::bg_flags::HAS_EXTENDED, "ext": 0,
            "url": url_id, "chars": []
        });
        assert_eq!(line(&[raw]).get_cell(0).unwrap().extended.url_id(), url_id);
    }
}
