// Copyright (c) 2026 xterm-rust contributors. Licensed under MIT.
// Shared fixture decoding keeps the differential test about buffer behavior.
use std::{cell::RefCell, rc::Rc};
use xterm_core::{
    buffer::{AttributeData, Buffer, BufferLine, CellData, ExtendedAttrs},
    options::{OptionValue, OptionsService, WindowsPty},
};

#[derive(Debug)]
pub struct LineSnapshot<'a> {
    pub identity: usize,
    pub wrapped: bool,
    pub cells: &'a str,
    pub full: &'a str,
    pub trimmed: &'a str,
}
#[derive(Debug)]
pub struct Snapshot<'a> {
    pub state: Vec<i64>,
    pub lines: Vec<LineSnapshot<'a>>,
    pub markers: Vec<(i64, bool)>,
}
#[derive(Debug)]
pub struct Case<'a> {
    pub id: usize,
    pub title: &'a str,
    pub options: Vec<&'a str>,
    pub before: Snapshot<'a>,
    pub cols: usize,
    pub rows: usize,
    pub after: Snapshot<'a>,
    pub events: &'a str,
}
fn numbers(text: &str) -> Vec<i64> {
    text.split('\t').map(|n| n.parse().unwrap()).collect()
}
fn snapshot<'a>(rows: &mut impl Iterator<Item = &'a str>) -> Snapshot<'a> {
    let row = rows.next().unwrap();
    let (_, state) = row.split_once('\t').unwrap();
    let state = numbers(state);
    let mut lines = Vec::new();
    for _ in 0..usize::try_from(state[14]).unwrap() {
        let row: Vec<_> = rows.next().unwrap().split('\t').collect();
        assert_eq!(row[0], "line");
        lines.push(LineSnapshot {
            identity: row[1].parse().unwrap(),
            wrapped: row[2] == "1",
            cells: row[3],
            full: row[4],
            trimmed: row[5],
        });
    }
    let (kind, text) = rows.next().unwrap().split_once('\t').unwrap();
    assert_eq!(kind, "markers");
    let markers = if text == "-" {
        Vec::new()
    } else {
        text.split('|')
            .map(|m| {
                let (line, disposed) = m.split_once(',').unwrap();
                (line.parse().unwrap(), disposed == "1")
            })
            .collect()
    };
    Snapshot {
        state,
        lines,
        markers,
    }
}
pub fn cases(fixture: &str) -> Vec<Case<'_>> {
    let mut rows = fixture.lines();
    let mut cases = Vec::new();
    while let Some(row) = rows.next() {
        let head: Vec<_> = row.split('\t').collect();
        assert_eq!(head[0], "case");
        let options = rows.next().unwrap().split('\t').skip(1).collect();
        let before = snapshot(&mut rows);
        let resize: Vec<_> = rows.next().unwrap().split('\t').collect();
        assert_eq!(resize[0], "resize");
        let after = snapshot(&mut rows);
        let (kind, events) = rows.next().unwrap().split_once('\t').unwrap();
        assert_eq!(kind, "events");
        cases.push(Case {
            id: head[1].parse().unwrap(),
            title: head[2],
            options,
            before,
            cols: resize[1].parse().unwrap(),
            rows: resize[2].parse().unwrap(),
            after,
            events,
        });
    }
    cases
}
fn decode_cell(text: &str) -> CellData {
    let fields: Vec<_> = text.splitn(6, ',').collect();
    let combined_data = if fields[5] == "-" {
        Vec::new()
    } else {
        fields[5]
            .split(',')
            .map(|s| u16::from_str_radix(s, 16).unwrap())
            .collect()
    };
    CellData {
        content: fields[0].parse().unwrap(),
        attributes: AttributeData {
            fg: fields[1].parse().unwrap(),
            bg: fields[2].parse().unwrap(),
            extended: ExtendedAttrs::new(fields[3].parse().unwrap(), fields[4].parse().unwrap()),
        },
        combined_data,
    }
}
pub fn hydrate(case: &Case<'_>) -> (Buffer, Vec<Rc<RefCell<BufferLine>>>) {
    let state = &case.before.state;
    let unsigned = |index: usize| usize::try_from(state[index]).unwrap();
    let options = Rc::new(OptionsService::default());
    for (key, value) in [
        ("scrollback", case.options[0]),
        ("tabStopWidth", case.options[1]),
    ] {
        options
            .set_raw(key, OptionValue::Number(value.parse().unwrap()))
            .unwrap();
    }
    options
        .set_raw(
            "reflowCursorLine",
            OptionValue::Bool(case.options[2] == "1"),
        )
        .unwrap();
    options
        .set_raw(
            "windowsPty",
            OptionValue::object(WindowsPty {
                backend: (case.options[3] != "-").then(|| case.options[3].to_owned()),
                build_number: (case.options[4] != "-").then(|| case.options[4].parse().unwrap()),
            }),
        )
        .unwrap();
    let mut buffer = Buffer::new(state[4] == 1, unsigned(0), unsigned(1), options).unwrap();
    buffer.host_dimensions.cols.set(unsigned(2));
    buffer.host_dimensions.rows.set(unsigned(3));
    buffer.lines.set_max_length(unsigned(5)).unwrap();
    let mut identities = Vec::new();
    for expected in &case.before.lines {
        let cells: Vec<_> = if expected.cells == "-" {
            Vec::new()
        } else {
            expected.cells.split('|').map(decode_cell).collect()
        };
        let mut line = BufferLine::new(cells.len(), None, expected.wrapped);
        for (x, cell) in cells.iter().enumerate() {
            line.set_cell(x, cell);
        }
        let line = Rc::new(RefCell::new(line));
        buffer.lines.push(line.clone()).unwrap();
        identities.push(line);
    }
    buffer.x = unsigned(6);
    buffer.y = unsigned(7);
    buffer.ybase = unsigned(8);
    buffer.ydisp = unsigned(9);
    buffer.saved_x = unsigned(10);
    buffer.saved_y = state[11];
    buffer.scroll_top = unsigned(12);
    buffer.scroll_bottom = unsigned(13);
    for (line, disposed) in &case.before.markers {
        assert!(!disposed, "input contains only active marker references");
        buffer.add_marker(*line);
    }
    (buffer, identities)
}
pub fn encode_cells(line: &BufferLine) -> String {
    if line.is_empty() {
        return "-".into();
    }
    (0..line.len())
        .map(|x| {
            let cell = line.get_cell(x).unwrap();
            let combined = if cell.combined_data.is_empty() {
                "-".into()
            } else {
                cell.combined_data
                    .iter()
                    .map(|c| format!("{c:x}"))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            format!(
                "{},{},{},{},{},{}",
                cell.content,
                cell.attributes.fg,
                cell.attributes.bg,
                cell.attributes.extended.ext(),
                cell.attributes.extended.url_id(),
                combined
            )
        })
        .collect::<Vec<_>>()
        .join("|")
}
pub fn assert_snapshot(
    buffer: &Buffer,
    identities: &[Rc<RefCell<BufferLine>>],
    expected: &Snapshot<'_>,
    label: &str,
) {
    let signed = |value: usize| i64::try_from(value).unwrap();
    let state = [
        signed(buffer.cols()),
        signed(buffer.rows()),
        signed(buffer.host_dimensions.cols.get()),
        signed(buffer.host_dimensions.rows.get()),
        i64::from(buffer.has_scrollback()),
        signed(buffer.lines.max_length()),
        signed(buffer.x),
        signed(buffer.y),
        signed(buffer.ybase),
        signed(buffer.ydisp),
        signed(buffer.saved_x),
        buffer.saved_y,
        signed(buffer.scroll_top),
        signed(buffer.scroll_bottom),
        signed(buffer.lines.len()),
    ];
    let mut expected_state = expected.state.clone();
    // The fixture retains the immutable constructor flag. The public source
    // getter additionally requires capacity greater than the current rows.
    expected_state[4] = i64::from(expected.state[4] == 1 && expected.state[5] > expected.state[1]);
    assert_eq!(state.as_slice(), expected_state, "{label}: state");
    for (y, expected) in expected.lines.iter().enumerate() {
        let line = buffer.lines.get(y).unwrap();
        let actual_id = identities
            .iter()
            .position(|l| Rc::ptr_eq(l, &line))
            .map_or(0, |i| i + 1);
        assert_eq!(actual_id, expected.identity, "{label}: line {y} identity");
        let line = line.borrow();
        assert_eq!(
            line.is_wrapped, expected.wrapped,
            "{label}: line {y} wrapped"
        );
        assert_eq!(
            encode_cells(&line),
            expected.cells,
            "{label}: line {y} cells"
        );
        for (trim, text) in [(false, expected.full), (true, expected.trimmed)] {
            let value = line.translate_to_string(trim, None, None, None);
            let actual = if value.is_empty() {
                "-".into()
            } else {
                value
                    .iter()
                    .map(|unit| format!("{unit:x}"))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            assert_eq!(actual, text, "{label}: line {y} translation trim={trim}");
        }
    }
}
