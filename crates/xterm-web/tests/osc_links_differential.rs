use xterm_core::buffer::{BufferLine, CellData, CharData};
use xterm_web::links::osc_links;

#[test]
fn source_seeded_cell_id_and_wrap_cases() {
    for (case, record) in include_str!("fixtures/osc-links.tsv").lines().enumerate() {
        let fields: Vec<_> = record.split('\t').collect();
        assert_eq!(fields.len(), 4, "case {case}");
        let y = fields[0].parse().unwrap();
        let allow = fields[1] == "1";
        let lines: Vec<_> = fields[2]
            .split('|')
            .map(|row| {
                let (wrapped, cells) = row.split_once(';').unwrap();
                let cells: Vec<_> = cells.split(',').collect();
                let mut line = BufferLine::new(cells.len(), None, wrapped == "1");
                for (x, data) in cells.iter().enumerate() {
                    let data: Vec<u32> = data.split(':').map(|v| v.parse().unwrap()).collect();
                    let mut cell = CellData::from_char_data(&CharData {
                        attr: 0,
                        chars: if data[0] == 0 {
                            vec![]
                        } else {
                            vec![data[0] as u16]
                        },
                        width: data[2] as i32,
                        code: Some(data[0]),
                    });
                    cell.extended.set_url_id(data[1]);
                    cell.update_extended();
                    line.set_cell(x, &cell);
                }
                line
            })
            .collect();
        let uris = [
            None,
            Some("https://example.com/1"),
            Some("http://example.com/2"),
            Some("javascript:alert(1)"),
            Some("not a url"),
            Some(""),
            Some("mailto:x@example.com"),
        ];
        // This finite oracle corpus uses only the URIs above. Real arbitrary URI
        // validation belongs to the browser adapter's WHATWG Url implementation.
        let result = osc_links(
            &lines,
            y,
            |id| uris[id as usize].map(|uri| uri.encode_utf16().collect()),
            allow,
            |uri| {
                matches!(
                    String::from_utf16(uri).unwrap().as_str(),
                    "https://example.com/1" | "http://example.com/2"
                )
            },
        );
        let encoded = result
            .map(|links| {
                links
                    .iter()
                    .map(|link| {
                        let text: String = link
                            .text
                            .iter()
                            .map(|unit| *unit as u8)
                            .collect::<Vec<_>>()
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect();
                        format!(
                            "{text},{},{},{},{}",
                            link.range.start.x,
                            link.range.start.y,
                            link.range.end.x,
                            link.range.end.y
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(";")
            })
            .unwrap_or_else(|| "-".into());
        assert_eq!(encoded, fields[3], "case {case}: {record}");
    }
}
