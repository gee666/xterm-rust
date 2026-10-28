//! Decoder for the independent JS serializer's lossless, source-cell TSV format.
use std::collections::BTreeMap;
use xterm_core::buffer::{AttributeData, CellData, ExtendedAttrs};
use xterm_core::serialize::{
    BufferAccess, CellRange, HtmlConfig, HtmlOptions, HtmlRange, LineRange, MouseTracking,
    Position, SerializeError, SerializeModes, SerializeOptions,
};

#[derive(Default)]
pub struct Buffer {
    pub cells: Vec<Vec<CellData>>,
    pub wrapped: Vec<bool>,
    pub cursor: [usize; 3],
}
impl BufferAccess for Buffer {
    fn len(&self) -> usize {
        self.cells.len()
    }
    fn line_len(&self, row: usize) -> Result<Option<usize>, SerializeError> {
        Ok(self.cells.get(row).map(Vec::len))
    }
    fn cell(&self, row: usize, col: usize) -> Result<Option<CellData>, SerializeError> {
        Ok(self.cells.get(row).and_then(|line| line.get(col)).cloned())
    }
    fn is_wrapped(&self, row: usize) -> Result<bool, SerializeError> {
        Ok(self.wrapped.get(row).copied().unwrap_or(false))
    }
    fn cursor_x(&self) -> usize {
        self.cursor[0]
    }
    fn cursor_y(&self) -> usize {
        self.cursor[1]
    }
    fn base_y(&self) -> usize {
        self.cursor[2]
    }
}

pub struct Case {
    pub id: usize,
    pub kind: String,
    pub cols: usize,
    pub rows: usize,
    pub active_alternate: bool,
    pub attrs: AttributeData,
    pub modes: SerializeModes,
    pub margins: [usize; 2],
    pub ansi: SerializeOptions,
    pub html: HtmlOptions,
    pub selection: Option<CellRange>,
    pub config: HtmlConfig,
    pub expected: Result<Vec<u16>, String>,
    pub normal: Buffer,
    pub alternate: Buffer,
}

fn num<T: std::str::FromStr>(value: &str) -> T
where
    T::Err: std::fmt::Debug,
{
    value.parse().unwrap()
}
fn opt<T: std::str::FromStr>(value: &str) -> Option<T>
where
    T::Err: std::fmt::Debug,
{
    (value != "-").then(|| num(value))
}
fn units(value: &str) -> Vec<u16> {
    if value.is_empty() {
        Vec::new()
    } else {
        value
            .split('.')
            .map(|unit| u16::from_str_radix(unit, 16).unwrap())
            .collect()
    }
}
fn text(value: &str) -> String {
    String::from_utf16(&units(value)).unwrap()
}
fn attributes(value: &str) -> AttributeData {
    let columns: Vec<_> = value.split('/').collect();
    assert_eq!(columns.len(), 4);
    AttributeData {
        fg: num(columns[0]),
        bg: num(columns[1]),
        extended: ExtendedAttrs::new(num(columns[2]), num(columns[3])),
    }
}
fn cell(value: &str) -> CellData {
    let columns: Vec<_> = value.split('/').collect();
    assert_eq!(columns.len(), 6);
    CellData {
        content: num(columns[0]),
        attributes: AttributeData {
            fg: num(columns[1]),
            bg: num(columns[2]),
            extended: ExtendedAttrs::new(num(columns[3]), num(columns[4])),
        },
        combined_data: units(columns[5]),
    }
}
fn pair(value: &str) -> [usize; 2] {
    value
        .split(',')
        .map(num)
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}
fn modes(bits: u32, mouse: &str) -> SerializeModes {
    let enabled = |index: u32| bits & (1_u32 << index) != 0;
    SerializeModes {
        application_cursor_keys: enabled(0),
        application_keypad: enabled(1),
        bracketed_paste: enabled(2),
        insert: enabled(3),
        origin: enabled(4),
        reverse_wraparound: enabled(5),
        send_focus: enabled(6),
        wraparound: enabled(7),
        show_cursor: enabled(8),
        mouse_tracking: match mouse {
            "none" => MouseTracking::None,
            "x10" => MouseTracking::X10,
            "vt200" => MouseTracking::Vt200,
            "drag" => MouseTracking::Drag,
            "any" => MouseTracking::Any,
            other => panic!("unknown mouse {other}"),
        },
    }
}
fn ansi(value: &str) -> SerializeOptions {
    let c: Vec<_> = value.split(',').collect();
    assert_eq!(c.len(), 5);
    SerializeOptions {
        range: opt(c[0]).map(|start| LineRange {
            start,
            end: num(c[1]),
        }),
        scrollback: opt(c[2]),
        exclude_modes: c[3] == "1",
        exclude_alt_buffer: c[4] == "1",
    }
}
fn html(value: &str) -> HtmlOptions {
    let c: Vec<_> = value.split(',').collect();
    assert_eq!(c.len(), 6);
    HtmlOptions {
        scrollback: opt(c[0]),
        only_selection: c[1] == "1",
        include_global_background: c[2] == "1",
        range: opt(c[3]).map(|start_line| HtmlRange {
            start_line,
            end_line: num(c[4]),
            start_col: num(c[5]),
        }),
    }
}
fn selection(value: &str) -> Option<CellRange> {
    if value == "-" {
        return None;
    }
    let c: Vec<_> = value.split(',').map(num).collect();
    assert_eq!(c.len(), 4);
    Some(CellRange {
        start: Position { x: c[0], y: c[1] },
        end: Position { x: c[2], y: c[3] },
    })
}

pub fn load() -> Vec<Case> {
    let mut palettes = BTreeMap::new();
    let mut cases: Vec<Case> = Vec::new();
    for row in include_str!("../fixtures/serialize-oracle.tsv").lines() {
        let c: Vec<_> = row.split('\t').collect();
        match c[0] {
            "palette" => {
                let colors: [String; 256] = c[2]
                    .split(',')
                    .map(text)
                    .collect::<Vec<_>>()
                    .try_into()
                    .unwrap();
                assert!(palettes.insert(num::<usize>(c[1]), colors).is_none());
            }
            "case" => {
                assert_eq!(c.len(), 20);
                let id = num(c[1]);
                assert_eq!(id, cases.len());
                let expected = if c[18] == "ok" {
                    Ok(units(c[19]))
                } else {
                    assert_eq!(c[18], "error");
                    Err(text(c[19]))
                };
                cases.push(Case {
                    id,
                    kind: c[2].into(),
                    cols: num(c[3]),
                    rows: num(c[4]),
                    active_alternate: c[5] == "1",
                    attrs: attributes(c[6]),
                    modes: modes(num(c[7]), c[8]),
                    margins: pair(c[9]),
                    ansi: ansi(c[10]),
                    html: html(c[11]),
                    selection: selection(c[12]),
                    config: HtmlConfig {
                        font_family: units(c[13]),
                        font_size: num(c[14]),
                        foreground: (c[15] != "-").then(|| units(c[15])),
                        background: (c[16] != "-").then(|| units(c[16])),
                        ansi_colors: opt::<usize>(c[17])
                            .map(|id| palettes.get(&id).unwrap().clone()),
                    },
                    expected,
                    normal: Buffer::default(),
                    alternate: Buffer::default(),
                });
            }
            "buffer" => {
                assert_eq!(c.len(), 6);
                let case = cases.last_mut().unwrap();
                let buffer = if c[1] == "N" {
                    &mut case.normal
                } else {
                    assert_eq!(c[1], "A");
                    &mut case.alternate
                };
                buffer.cursor = [num(c[2]), num(c[3]), num(c[4])];
                let length = num(c[5]);
                buffer.cells.resize_with(length, Vec::new);
                buffer.wrapped.resize(length, false);
            }
            "line" => {
                assert_eq!(c.len(), 5);
                let case = cases.last_mut().unwrap();
                let buffer = if c[1] == "N" {
                    &mut case.normal
                } else {
                    assert_eq!(c[1], "A");
                    &mut case.alternate
                };
                let row = num::<usize>(c[2]);
                buffer.wrapped[row] = c[3] == "1";
                buffer.cells[row] = c[4].split('|').map(cell).collect();
                assert_eq!(buffer.cells[row].len(), case.cols);
            }
            other => panic!("unknown fixture row {other}"),
        }
    }
    cases
}
